---
layout: post
title: Safely running a public IPFS gateway using nginx
---

-----

# TL;DR;

- publish multiple websites via IPFS
- provide access via IPFS gateway
- prevent download of arbitrary, non-endorsed content

# Why?

If you are working or playing with IPFS, you are probably running an ipfs node somewhere on the internet. IPFS has the nice property that your node will only ever contain data that you have requested or added yourself, so this is a pretty safe thing to do. The bandwidth use of an ipfs node hosting some obscure content is pretty moderate, and you won't suddenly find illegal content in your local cache.

But once you expose the ipfs gateway to the world, this goes out of the window. *Anybody* can request *anything* that is *anywhere* on IPFS via your gateway. Currently this is not such a big deal, since IPFS is still relatively unknown. But even now it is not wise to rely on security by obscurity.

But on the other hand, it would be nice to expose an IPFS gateway to provide quick and safe access to *your own* content or content you endorse. So how do you do this?

# DNS links

## Assigning hashes to domain names

Ipfs allows assigning IPFS hashes (or IPNS names) to DNS entries via TXT records with content `"dnslink=/ipfs/QmRzNhBJd1ppKCNkXe1V7qqTC1yx72aVzMZDrbM8QVUcFj"`. Once you have such a TXT record pointing to a valid hash, you can use it in an ipns url. E.g. `http://gateway.ipfs.io/ipns/blog.klaehn.org`.

To see how this works, you can investigate sites published on ipfs via [`dig`](https://linux.die.net/man/1/dig)

```shell
dig -t TXT blog.klaehn.org
```

will return hash of the *current* content of this blog.

# Assigning IP addresses to domains.

Assigning the IP address of your IPFS gateway to your domain name works exactly as with any other web server. E.g. 

```shell
dig blog.klaehn.org
```

will return the IP address of my IPFS gateway.

## Virtual hosting

In addition, when an IPFS gateway is accessed, it will look at the [Host header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Host), and check if there is a corresponding DNS TXT record. If there is, the gateway will serve the corresponding name or hash.

You can think of this as old-fashioned [virtual hosting](https://en.wikipedia.org/wiki/Virtual_hosting). One gateway will serve different content based on the `Host` header.

# Setting up NGINX as a proxy

[NGINX](https://www.nginx.com/) is a high performance web server and proxy server. It is pretty easy to set up.

I am using ubuntu, so all I had to do to get started was `apt install nginx`. See e.g. [this howto](https://www.digitalocean.com/community/tutorials/how-to-install-nginx-on-ubuntu-16-04) for details. You will definitely find something similar for your distro of choice.

To use nginx as a proxy for the IPFS gateway, you just need to add a single proxy_pass directive to the /etc/nginx/hosts-enabled/default file. To enable the resolution based on dnslink, you just need to make sure that the `Host` header is forwarded.

```
server {
  ...
  location / {
    proxy_pass http://127.0.0.1:8080;
    proxy_set_header Host            $host;
    proxy_set_header X-Forwarded-For $remote_addr;
  }
  ...
}
```

# Preventing arbitrary content download

With the above proxy setup, you can point any number of your domains to your gateway. The gateway will then look up the dns TXT record corresponding to the domain and serve the linked content.

But what if somebody accesses your gateway via its raw IP address? In that case it is still possible to request arbitrary content via the gateway.

`http://<your gateway ip>/ipfs/<some 4GB warezed movie>`

**Not good**.

So let's prevent that. The first thing I thought of was to just disallow access via `<your gateway ip>/ipfs` or `<your gateway ip>/ipns`.

```
server {
  location /ipns/ {
    return 404;
  }

  location /ipfs/ {
    return 404;
  }
}
```

## Is it safe now?

Now it is no longer *easy* for people to access arbitrary content via your gateway. But there is still something they could do. It might be a bit far-fetched, but somebody could create his own DNS TXT record and A record pointing to your IP address to download arbitrary content, or just create the DNS TXT record and fake the host header. To prevent that as well, you would have to configure name based virtual hosting in nginx to prevent arbitrary names to be forwarded to the IPFS gateway.

The easiest way is to just return 404 when the site is accessed *in any way* via its IP address without the Host header.

```
server {
  ...
  location / {
    return 404;
  }
  ...
}
```

Then have separate site files for each site you want to serve via the gateway. These have to be created in `/etc/nginx/sites-available` and symlinked in `/etc/nginx/sites-enabled`.

A simple config that just does the forwarding for `your.domain` would look like this:

`/etc/nginx/sites-available/your.domain`

```
server {
  listen 80;
  listen [::]:80;

  server_name *.your.domain;

  location / {
    proxy_pass http://127.0.0.1:8080;
    proxy_set_header Host            $host;
    proxy_set_header X-Forwarded-For $remote_addr;
  }
}
```

# To do

One thing that is missing is setting up encryption via [letsencrypt](https://letsencrypt.org/). This is probably just some additional work with nginx. I might describe this in a later blog post. 

# Result

- set up ipfs with gateway on your node, but protect 8080 via firewall
- use NGINX to proxy port 8080 on port 80, while forwarding the Host header
- serve any number of your domains, while preventing IPFS from serving arbitrary content
- create DNS A records to point to your public gateway
- create DNS TXT records containing dnslink every time you want to update your content

