# Beaker Whitepaper

## Abstract
Small summary of the contents of the whitepaper

## Introduction

This whitepaper outlines the Beaker kernel and operating system.

## Risks, Issues, and Problems of Smart Contract Systems

This will outline what the issues we're trying to solve are.

## Requirements

This sets out what we need to secure, and what tools we need to do that.  It
will conclude noting that we need a form of control or introspection into the
running systems. This will dovetail with the next section.

### Introspection

This will outline in more detail how having some control and insight into what
is happening as the system is executing. E.g. it allows us to see what each
chunk of code is doing to the state of the system, and possibly disallow that if
necessary.

### Control

This will section will focus how introspection and control is only possible if we establish restrictions over the execution of the system via an operating system.

## Kernel

This setion outlines how we build an operating system kernel (i.e. procedures and
syscalls) and how this gives us the control needed to implement a number of
different security systems.

### Procedures

Here we outline procedures, how they would be created, deployed and how they need a system call interface to be able to interact with the kernel.

### System Calls

This will outline how system calls work.

#### How We Can't Implement System Calls

Present the limitations of things like static call, and the difficulty of
coroutines.

#### How We Can Implement System Calls

Present the solution using delegate call and on chain code verification. It
would be nicer if on chain code verification had more prominence, but this makes
the most sense when reading the whitepaper.

## Security Model

This section builds on the operating system concepts and adds the security
functionality via capabilities.

### Capability Based Security and Their Advantages

Not just what capabilities are, but why we have chosen them over the
alternatives.

### Implementing a Capability Based Security Model

This section will outline the capability model we have designed, and how it uses all of the about material to enforce its model.

### Kernel Objects and Capabilities
Here we list through all the kernel objects required to implement our security model and thier corresponding capabilities

#### Procedure Table
Here we descibe the procedure table as an object, and how it can be changed. 
We can elaborate that procedures themselves are objects and require a capability to be accessed.

#### Capability Table
Here we describe the capability table as an object, and how it can be changed.

#### Storage
Here we describe storage as an object, and how it can be changed.

#### Events
Here we describe events as an object, and how it can be changed.

#### Gas 
Here we describe gas as an object, and how it can be changed.

## Applications

Some information on "usage characteristics". How do you actually use this
properly. This should be high level and not necessarily include code examples or
the like.

### Filesystem
Here we can provide an example of how a filesystem that uses the capability model can provide a storage abstraction to procedures

## Conclusion

Summarise what we have outlined above and why it is useful.
