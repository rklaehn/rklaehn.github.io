---
layout: post
title: Publishing a blog on IPFS
comments: false
---

-----

# TL;DR;

- [jekyll](https://jekyllrb.com/) based blog
- hosted on [github](github.com)
- continuous integration using [travis ci](https://travis-ci.org/)
- pushed to IPFS by travis
- DNS by [aws route53](https://aws.amazon.com)

# Why?

My [blog](rklaehn.github.io) is currently hosted on
github using jekyll and poole. A blog without comments
is static content which can be easily hosted on the
[interplanetary file system](https://ipfs.io/).

I always thought it would be fun to do this, but the
recent ownership change of github gave me additional
motivation.

The steps to get there are all reasonably easy. This is
not rocket science. But there are quite a few steps to 
get a very smooth experience, so I thought it would be
useful to document this.

# Prerequisites

You need an ipfs node that is under your control. I am
running one on a tiny *free tier* AWS EC2 instance, but
anything else will do as well, provided that it has
ssh access.

Here are some instructions on how to set up a server:
- [Installing ipfs](https://flyingzumwalt.gitbooks.io/decentralized-web-primer/content/install-ipfs/)
- [How to Host Your IPFS Files Online Forever
](https://medium.com/@merunasgrincalaitis/how-to-host-your-ipfs-files-online-forever-f0c56b9b5398)

# Static content generation

This howto will work for *any website* that can be
statically generated using common open source tools.

I am using a [jekyll](https://jekyllrb.com/) based
blog using [poole](https://github.com/poole/poole).

To generate the blog locally, all that is needed is to
invoke `jekyll build` in the root of the repo. This will
output the site in the `./_site` subdirectory.

On travis CI, it works exactly the same, except you of
course have to make sure to install jekyll and all
needed dependencies. I have not written a single line of
ruby in my life, but it seems that how you do it is to
have a file called `Gemfile` in the root of the repo.

```ruby
source "https://rubygems.org"

gem "jekyll"
gem "html-proofer"
gem "jekyll-paginate"
gem "redcarpet"
```

To get travis to install these deps, you just need to
set the language:

```yaml
language: ruby
cache: bundler
rvm:
- 2.4.1
```

# Publishing to your IPFS node

Once you have generated your content and maybe run some
checks on the generated content using tools like [html-proofer](https://github.com/gjtorikian/html-proofer),
it is time to deploy the output.

The output has to be added *and pinned* on **your** ipfs
node to be continuously statically available. If you
would do this manually you would scp the data to your
node and run `ipfs add <data>` there.

There are many different ways to automate this. I am
just opening a tunnel from the travis ci job to the ipfs
node and then use the ipfs cli to add the data *as if it
was local*.

Compressing and piping the data via ssh is more
efficient, but also more complex.

## Getting the ipfs binary

This part of the deploy script gets the ipfs binary and
puts it on the path.

```shell
# get ipfs (version must match what you have on your node
wget -qO- https://dist.ipfs.io/go-ipfs/v0.4.15/go-ipfs_v0.4.15_linux-amd64.tar.gz | tar xz
# get the ipfs binary on the path
PATH=./go-ipfs/:$PATH
```

The IPFS binary needs to be the same version as what
is running on your node.

## Setting up safe ssh connection to the ipfs node

The free version of travis ci does not allow configuring
ssh keys via the gui. So we need to manually generate a
keypair, encrypt it, and add the encrypted private key
to the repo. The public key needs to be added to the 
ipfs node.

The exact details can be found in this excellent [blog post](https://oncletom.io/2016/travis-ssh-deploy/).

Here is the relevant part of the .travis.yml file that
decrypts the key and adds it to ssh.

```yaml
after_success:
  - openssl aes-256-cbc -K $encrypted_<...>_key -iv $encrypted_<...>_iv -in deploy_rsa.enc -out /tmp/deploy_rsa -d
  - eval "$(ssh-agent -s)"
  - chmod 600 /tmp/deploy_rsa
  - ssh-add /tmp/deploy_rsa
  - ./script/deploy
```

## Opening the API tunnel to your node

Once you have configured ssh, you can open the tunnel.
We want to use the ipfs api, so we need to forward port
5001 or whatever is configured as the api port on your
ipfs node.

```shell
ssh -N -L 5001:localhost:5001 user@your.ipfs.node &
```

## The actual publishing

After all this setup ceremony, we can use ipfs from
the deploy script *as if it was running locally*. This
is great, since if we had to do some more complex
interactions with ipfs from the deploy script, there
would be no additional effort.

Here is the code from the deploy script to set do the
actual publishing:

```shell
HASH=`ipfs --api /ip4/127.0.0.1/tcp/5001 add -r -Q _site`
```

We need the hash for further processing, so we run the
ipfs add with -Q so the output just contains the hash.
We also provide the --api parameter so ipfs is using
the api we forwarded via ssh, and is not looking for a
local repo in `~/.ipfs`.

# Updating the DNS TXT record

Now we have published the static content. It is
available both from our node and from any ipfs gateway
via e.g. `https://gateway.ipfs.io/ipfs/<hash>`. But we
of course want the blog to be available via a friendly
name.

In many IPFS howtos, it is recommended to use IPNS to
do this. See this [example](https://flyingzumwalt.gitbooks.io/decentralized-web-primer/content/publishing-changes/) from the decentralized web primer, or
[this](https://medium.com/coinmonks/how-to-add-site-to-ipfs-and-ipns-f121b4cfc8ee) blog post.

But this has not been working very well for me.
Both IPNS lookup and IPNS publishing is slow and
unreliable for me. YMMV.

So what I am doing instead is updating a dns TXT entry
with the new IPFS hash that was generated when
uploading the data.

I realize that this is not fully decentralized, but
it works really well for me. I will switch to IPNS as
soon as it is fast and reliable.

I am using AWS Route53, but this will of course work
with any DNS provider that allows API access.

## Getting a hosted zone

To use AWS route 53, you obviously need to have a domain
registered with AWS. Once you have that, you will need
to create a so-called [hosted zone](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/CreatingHostedZone.html).
This is very well documented and can all be done via the
[aws management console](https://aws.amazon.com/console/)

Note down the hosted zone ID for the next step.

## Enabling programmatic access

### Creating IAM Policy

Fine grained access control in AWS is done via policies.
Giving a travis job programmatic access to AWS is a bit
scary, so we want to give travis the *absolute minimum*
rights to update a DNS record, and nothing else.

We need a policy that allows just the action [`route53:ChangeResourceRecordSets`](https://docs.aws.amazon.com/Route53/latest/APIReference/API_ChangeResourceRecordSets.html),
but *only* for the hosted zone we created above.

The final policy JSON should look something like this:

```javascript
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "VisualEditor0",
            "Effect": "Allow",
            "Action": "route53:ChangeResourceRecordSets",
            "Resource": "arn:aws:route53:::hostedzone/<YourHostedZone>"
        }
    ]
}
```

### Creating IAM User

To allow updating the dns entry from travis, you have
to create a *separate* AWS IAM user with minimal
privileges. This can be done using [IAM](https://console.aws.amazon.com/iam/home?region=us-east-1#/users).
Give the user just *Programmatic access*. Assign the
policy created in the previous step, and **nothing else**.

You will be prompted to download a .csv file containing
the access credentials. Store it somewhere safe, since
this will be the only time you will be able do download
it.

### Configuring travis

To allow travis to talk to your AWS account, you need to
configure two environment variables. The `Access key ID`
value is stored in `AWS_ACCESS_KEY_ID`.
The `Secret access key` goes into `AWS_SECRET_ACCESS_KEY`.

Keep the `Display value in build log` toggle disabled
for the `AWS_ACCESS_KEY_ID` and *especially* the
`AWS_SECRET_ACCESS_KEY`.

To be able to use the aws cli from travis, you need to
install it from `.travis.yml`. It will pick up the AWS
account credential from the environment variables
configured above.

```yaml
before_install:
  - pip install --user awscli
  - export PATH=$PATH:$HOME/.local/bin
```

### The actual command

After all this ceremony, we can finally execute the
command to update the DNS TXT record. You can use the
aws cli tools to generate a JSON stub for a route53
`change-resource-record-sets` request, and then fill
in your data.

The snippet below assumes that `$HOSTEDZONE` contains
your hosted zone id, `$NAME` contains the name of the
TXT record *in the hosted zone* you want to update, and
`$HASH` contains the IPFS hash you want to update to.

```shell
REQUEST='{"HostedZoneId":"'$HOSTEDZONE'","ChangeBatch":{"Comment":"Updating '$NAME' to '$HASH'","Changes":[{"Action":"UPSERT","ResourceRecordSet":{"Name":"'$NAME'","Type":"TXT","TTL":30,"ResourceRecords":[{"Value":"\"dnslink='$HASH'\""}]}}]}}'
aws route53 change-resource-record-sets --cli-input-json "$REQUEST"
```

# Adding an A or CNAME DNS record

To allow resolving your blog directly via its name,
you have to add additional DNS records.

## Running your own public IPFS gateway

If you run your own public ipfs gateway, you just need
to create an [A record](https://en.wikipedia.org/wiki/List_of_DNS_record_types)
that assigns the public ip address of your gateway to
the name of your blog.

## Using gateway.ipfs.io

If you don't want to run your own gateway, you can
either create an A record that points to a public ip
of `gateway.ipfs.io`, *or* you can create a CNAME record
for your domain that points to `gateway.ipfs.io`.

# Putting it all together

It is useful to have the CI publish to IPFS for every
commit, but we only want to update the DNS name for each
commit on the *master* branch.

To accomplish this, we need to check the `TRAVIS_BUILD`
travis environment variable to check if we are on
master, and in addition check the `TRAVIS_PULL_REQUEST`
environment variable to make sure that we are *actually*
on master and not just on a PR against master.

See the [travis docs](https://docs.travis-ci.com/user/environment-variables/) about environment variables.

So when we are not really on master, we just bail out
before publishing:

```shell
if [ "$TRAVIS_BRANCH" = "master" -a "$TRAVIS_PULL_REQUEST" = "false" ]; then
    NAME='blog.klaehn.org'
    echo "publishing to $NAME"
else
    NAME='blog-dev.klaehn.org'
    echo "Not on master. Aborting"
    exit 0
fi
```

Alternatively, you could publish to a different dns
entry in the same hosted zone, to have a preview
containing whatever was your last successful commit.

## Files

Here are all the files for the continuous integration

.travis.yml

```yaml
dist: trusty
sudo: false
before_install:
  - pip install --user awscli
  - export PATH=$PATH:$HOME/.local/bin
addons:
  ssh_known_hosts:
  - <ip-of-your-ipfs-node>
language: ruby
cache: bundler
rvm:
- 2.4.1
script: "./script/cibuild"
after_success:
  - openssl aes-256-cbc -K $encrypted_<...>_key -iv $encrypted_<...>_iv -in deploy_rsa.enc -out /tmp/deploy_rsa -d
  - eval "$(ssh-agent -s)"
  - chmod 600 /tmp/deploy_rsa
  - ssh-add /tmp/deploy_rsa
  - ./script/deploy

```

script/cibuild

```shell
#!/bin/bash
set -e # halt script on error
#
bundle exec jekyll build
```

script/deploy

```shell
#!/bin/bash
set -e # halt script on error
# get ipfs (version must match what you have on your node
wget -qO- https://dist.ipfs.io/go-ipfs/v0.4.15/go-ipfs_v0.4.15_linux-amd64.tar.gz | tar xz
PATH=./go-ipfs/:$PATH
# open tunnel to ipfs node
ssh -N -L 5001:localhost:5001 <user>@<ip-of-your-ipfs-node> &
# wait for some time for the tunnel to be established
sleep 10
# add jekyll output to ipfs
HASH=`ipfs --api /ip4/127.0.0.1/tcp/5001 add -r -Q _site`
echo $HASH
if [ "$TRAVIS_BRANCH" = "master" -a "$TRAVIS_PULL_REQUEST" = "false" ]; then
    NAME='blog.your.domain'
    echo "publishing to $NAME"
else
    echo "Not on master. Aborting"
    exit 0
fi
HOSTEDZONE='<your-hosted-zone-id>'
REQUEST='{"HostedZoneId":"'$HOSTEDZONE'","ChangeBatch":{"Comment":"Updating '$NAME' to '$HASH'","Changes":[{"Action":"UPSERT","ResourceRecordSet":{"Name":"'$NAME'","Type":"TXT","TTL":30,"ResourceRecords":[{"Value":"\"dnslink='$HASH'\""}]}}]}}'
echo $REQUEST
aws route53 change-resource-record-sets --cli-input-json "$REQUEST"
```

# Monitoring the build

To monitor the build, you can of course use the travis console. Once the DNS record is being updated, you can
check the current value using

```shell
watch dig -t TXT blog.your.domain
```

# Result

Now there are many ways to access your blog.

- Via any public ipfs gateway
https://gateway.ipfs.io/ipns/blog.klaehn.org

- Via your local ipfs daemon
https://localhost:8080/ipns/blog.klaehn.org

- Directly via the domain name
http://blog.klaehn.org

If you want to host your own blog on ipfs, feel free to use https://github.com/rklaehn/rklaehn.github.io as a starting point.
Just remove the blog posts. But you might be better off starting with a fresh clone of https://github.com/poole/poole and just
copying .travis.yml and the script directory.
