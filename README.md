# Cali

A "framework" for developing rust microservices.

Please note, cali is very new and not battle tested. It has a whole bunch of things it can't do yet and known issues that I'm still planning on fixing. That being said, it's built on some solid libraries that are stable, and you should be able to convert a cali project into it's constituent parts. See the TODO's at the bottom of the readme for more details on what's missing and what to expect.

# Getting started

## Prerequisites

Cali needs the `protoc` protocol buffers compiler + protocol buffer resource files to build tonic.

### Ubuntu
```
sudo apt install -y protobuf-compiler libprotobuf-dev
```

Then you want to go ahead and grab cali's cli tool:
```
cargo install cali_cli
```

Finally, create a new project with:
```
cali new <your project name>
```

This generates a cargo workspace with the following structure:
```
.
├── interface
│   └── grpc
│       ├── models
│       └── services
├── store
│   └── src
│       └── repositories
└── web
    ├── config
    └── src
        ├── controllers
        ├── entry
        └── protos
```
The web package is your entry point, and the interfaces directory specifies your GRPC services and models. Type out a simple service definition under `/interfaces/grpc/services` and run:

```
cali generate controllers
```

You should see your rust controllers generated in the controllers directory. From here you get to choose:

1. Write your code directly in the controller (for simple endpoints)
2. Create some modules directly in the web crate to handle your logic.
3. Add a new cargo library and have web depend on it.

For your database logic, we autogenerate a store crate and add it to your project directly. You can delete this if you don't need it. Part of the setup server macro creates a connection pool to your database and injects it into the cali context. You can embed your own global handles to connection pools, configs, or any shared atomic references. As a rule, keep the context light & avoid shared state & mutexes in there unless you understand exactly what you're doing!

Cali also includes the `Ensnare` derive macro, as a wrapper around your database models to codegen insert, update(TODO), and select(TODO) logic. Generally cali prefers sqlx as the highest layer of abstraction, but rewriting certain codepaths can be a chore, so you can lean on this to make life a bit easier.

Feel free to expand the `setup_server!()` macro, and have a look at what it does. You can take parts of the macro, or keep it as is. If you don't want all the database/config management, or you want to handle your tonic server more directly you can just manually implement the parts of main you like. Cali doesn't care about your main file. The `setup_server!()` macro will be split into smaller parts so you can pick and choose as you like.

## What is cali?

It's a series of codegen and convenience wrappers to write web applications in rust. At this point it only does GRPC, but the project is planned and structured to support other schema'd and schemaless protocols. I'd like to add some form of JSON and websockets.

## Why is cali?

Cali wanted to solve the following specific problems:

- Have a CLI that get's you up and running with a rust microservice instantly. (Kind of there, moving target)
- Schema based protocols allow you to codegen your request handlers, so that you don't have to write "controllers" that implement your service's trait, which you already did in your proto files. Just write your proto file, run a command and start writing your code. (WIP)
- Transport should not be intertwined with application logic, except when the application is for transport. A cali project tries to get you to write your web interface layer separate from your application logic. From there we aim to support multiple ways to communicate (sockets, streams, queues, etc.). There is a place where you handle your transport, and there's a place where you handle your application logic. It's not always trivial/possible to switch your transport, but your framework shouldn't find it's way into the discussion. (Only GRPC is supported now, so this is just fluff)
- Rust compile times are notorious, and not every team has the expertise/time/option to improve their crates' compilation time. The last resort usually ends up being crate splitting. So what if we just already separated our applications boundaries with crates? Cali makes it easier to split your application into different crates from the get-go, by providing a structure that makes sense. This is a opt in feature, and you are welcome to ignore it if you feel that you don't need it, or embrace it when the time comes. (Repository is split from main body, but core crates are still a concept)
- Provide a mechanism to to pass values however deep into your call stack without explicitly passing it around. This sounds really bad, and it can be, but humor me for a short detour. It's meant as a ergonomic last resort to pass certain values around without having to explicitly pass it as function parameters or embedding it in some wrapping struct and implementing functions on it. For example, you can embed the correlation/telemetry data in the context for a request, and then have that be available in some deeper nested code across crate boundaries, without having to pass the metadata around. Maybe you already have a good solution for this specific problem, but things like database connection pools, shared clients to third party services, handles to application config & more work really well with this model. It's advisable to not use this with mutable state as it could be hard to predict how requests compete for a mutex. This is the default mechanism for read only config, and to pass access to the database connection pool, it's also possible to still use cali without this mechanism if you don't feel comfortable with the idea of (tokio) task local storage. See the below section on How does it work? for more details. (This is done, but will be changed to allow extension by cali users, only used internally now)

## How is cali?

At this stage, cali has two main points:

### Codegen:
Most of the macro's are really just codegen for a series of dependencies

- Tokio: provides a fantastic async runtime
- Tonic: provides all the heavy lifting for handling GRPC.
- Tower: provides some necessary abstractions to facilitate transport independent middleware.
- Clap: provides a CLI interface for your web binary so that you can pass in args.
- Serde: Config file parsing, because you'll probably need to pass in some runtime secrets/config.

### Task locals
In your application, you'll probably want to have access to some global resources:

- A database connection pool
- Tracing headers like correlation ID's
- External services with their own connection pools, or single clients for rate limiting, etc.

You might have a few of these and feel comfortable passing them in explicitly, but this can quickly become messy. At the cost of losing some explicitness, cali introduces wrappers to create and wrap your tokio tasks with a cali context variable. This is simply a struct that holds a series of Arc's to your global instances and/or hold your request headers so that you can access them multiple layers deep, without having functions that have nothing to do with requests or database calls pass around variables just so that you can inject it in subsequent calls.

Other frameworks have built their own version of this concept, but what nice about cali's is that it's completely Tokio. Which means this mechanism isn't specific to the JSON or GRPC library that you use. You can reuse this with any other library, provided it can run on Tokio.

While this is really convenient, there is a cost associated with task switching, and task locals need to be moved around when you suspend your task. Holding onto a few references won't impact your applications performance significantly. Be aware that if you choose to crack this open for your own use case, that you ideally don't want to store too much in here, and always be very wary of shared global state, so try avoiding mutable wrappers like mutexes in this context.

## TODO's

- Add formatting to other issues.
- Flesh out the rest of snare for some "feels really nice" ORM goodness.
- Add postgres support to snare, lower priority
- Move some internal logic to internal crates to test crate splitting, lower priority
- A lot more, still have loads I'd like to experiment with, will add it here over time!
