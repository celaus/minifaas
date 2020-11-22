# MiniFaaS - A lightweight Function-as-a-Service Runtime

MiniFaaS is a multi-platform, no-fluff Function-as-a-Service runtime for home use. This means there are a few things MiniFaaS is **not**:

- A distributed system
- Protected against bad code (infinite loops, memory leaks, etc.)
- Enterprise-grade security and RBAC
- Hard to understand

In contrast to other FaaS runtimes, MiniFaaS aims to be:

- Simple run and maintain (e.g. via a single container)
- Reasonable performance
- Easy to understand and extend
- A home for small utility Functions

## Use cases and other visions

That's all very abstract, what do _you_ want to do with the runtime?

1. Check for updates for the containers running in my Nomad cluster
1. Fetch the current weather every couple of minutes from a [REST API](https://openweathermap.org) and show it on a [Neopixel display](https://blog.x5ff.xyz/blog/neosegment-pi-api/)
1. Reduce the number of containers running on my cluster
1. Integrate stuff with [Home Assistant](https://www.home-assistant.io)
1. Remote control [Azure Resources](https://blog.x5ff.xyz/blog/manage-container-instances-aci-functions/)

These are just some of my ideas, share yours!

# Features

So far, the function runtime has a minimal feature set to start with:

- HTTP trigger
- JavaScript/Typescript support via [Deno](https://deno.land)
- An ugly Web UI incl. code editor
- Actor-based multi-threaded async code execution
- CRUD for Function code via APIs/UI

## Planned

What's coming?

- CRON-like timer trigger
- Git support for storing Function code
- A better UI/API
- Tests 😅 and documentation; general code improvements
- Library support for JS
- More languages

# Contributing

If you want to help out, here are the things you could do (in order of commitment):

1. Open an issue (architecture discussion, feature requests, bugs, ...)
1. Spread the word via Hacker News, Reddit, ...
1. Write a blog post!
1. Plan/architect a feature
1. Improve the web frontend/create a design
1. Implement a feature or bug fix and PR


# License 

MIT