---
layout: post
title: Publishing static content on ipfs
comments: false
---

-----

# Publishing static content on IPFS

I have been a fan of IPFS for a long time. At [actyx.io](actyx.io), we are heavy users of IPFS, and I have been pushing for
its adoption. More on that at another time.

Now I was faced faced with the challenge of updating the hosting a few tiny static websites. It is pretty easy to use ipfs for
this, but doing so involves quite a few steps. So here is a step by step instruction to do it.

# Set up an ipfs node

To host data on ipfs, you need to run an ipfs node. Of course you can install ipfs locally on your development machine, but
that is not a really viable solution for hosting.

A frequent misconception about ipfs is that you can somehow push data to the network. That is not the case. An ipfs node will
only ever store data permanently that has been *explicitly* added or pinned to it. In addition, it will temporarily store data
that has been requested via its gateway. But that's it. This is a great decision, since it means that your node will only
serve *your* content. Nobody can abuse it to store illegal content or increase your bandwidth cost.

## Get a free tier ec2 instance

There are many different options for running an ipfs node. I decided to go with a "free tier" aws ec2 instance, since I am
familiar with aws. But any other server will do. IPFS is not very demanding. We run it on small android devices and raspberry
Pi at work.

## Configure your security group

In addition to the 

## Get and install ipfs-update

```
wget https://dist.ipfs.io/ipfs-update/v1.5.2/ipfs-update_v1.5.2_linux-amd64.tar.gz
tar -xzf ipfs-update_v1.5.2_linux-amd64.tar.gz
sudo install.sh
```

## Get latest ipfs

```
sudo ipfs-update install latest
```

## Configuring it as a service using systemd

```
```

## Adding the static content

## Installing nginx

## Setting up nginx as a proxy for your ipfs node

## Setting up DNS entries

### A record

Like with any other hosting option, you need to set the A record to assign an ip address to your domain.

### TXT record

To tell IPFS which key to serve, you have to create a TXT record with the content `"dnslink=/ipfs/<hash>"`, where hash is the
hash of the static content you want to serve. The details how to do this depend on the service you use.
