# Flair #

An opinionated GRPC/JSON Backend Web-framework for Rust.

## Goals ##
+ Easy to use CLI to create a new project, scaffold out components.
+ Simple decorate attribute on application main function to setup config, database connections, external service connections GRPC


## Structure Methodology ##

A typical flair project is split into application interfaces, and application logic. 

Application interfaces are concerned with how the application connects to the outside world.

Application logic is split into business logic contexts in the spirit of Domain Driven Design.


Flair takes an opinionated stance in a few regards, and I'll provide some short motivations on why certain decisions were made.

1. Flair splits every business logic unit into it's own crate. Being built on Rust, we are perpetually tied to language's issues. We get a great compiler, but a slow one. Optimising rust build times is not impossible, but it isn't trivial. Flair aims to make Rust more accessible to people coming from environment with hot reloading and rapid development times. We can't quite get there without significant effort. A simple yet effective solution is to just structure our projects into smaller units. These units move together, and by doing things this way I hope flair will make the swift from monolith -> to macroservice -> microservice (and back) easier. Your application is forced to be modular and composable. Hopefully this will assist in refactoring efforts (which if we face it, is inevitable), only time will tell.
2. Flair tries to make rust development less verbose. This is achieved via Rust's super powerful macro system. Consequently, most gory details are abstracted away from the user. Rust purests might miss that and the control that comes with that. I have and will try to allow some way to hook into this for those interested in having more control, optimizing for scale is a personal effort for every team and the framework will try to accomodate that, but when caught between a situation where some optimization might require adding complexity to the use of the framework, Flair will choose ease of use. That's not to say Flair doesn't aim to be performant, but if you are the one in a million company that needs that, maybe you should consider rolling your own setup.
3. Flair is meant to disprove the point that writing backend code is the sole domain of higher level languages. Rust is not the *best* language, but it's ideas are an improvement on the standard paradigms, and developers can learn a lot from using the language. It improves the way developers think, and will benefit you in the long run. Fight me.

