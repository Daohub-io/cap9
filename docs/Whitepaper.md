# Beaker Whitepaper

## Executive Summary

To be completed after the draft is complete.

## Introduction

This whitepaper outlines the Beaker kernel and operating system.

## Current Environment

### Blockchain, Ethereum, and Smart Contracts

This will set out that: blockchains exist, smart contracts exist and Ethereum is
an example of that. This is just a summary to prepare the reader and is low
priority.

### Risks, Issues, and Problems of Smart Contract Systems

This will outline what the issues we're trying to solve are.

## The Security Model Requirements

This sets out what we need to secure, and what tools we need to do that.  It
will conclude noting that we need a form of control or introspection into the
running systems. This will dovetail with the next section.

## Introspection via an Operating System

This setion outlines how we build an operating system (i.e. procedures and
syscalls) and how this gives us the control needed to implement a number of
different security systems.

### The Necessity and Advantage of Introspection and Control

This will outline in more detail how having some control and insight into what
is happening as the system is executing. E.g. it allows us to see what each
chunk of code is doing to the state of the system, and possibly disallow that if
necessary.

### Providing Introspection and Control

This will outline how system calls work.

#### Overview of System Calls

I.e. what are they?

#### How We Can't Implement System Calls

Present the limitations of things like static call, and the difficulty of
coroutines.

#### How We Can Implement System Calls

Present the solution using delegate call and on chain code verification. It
would be nicer if on chain code verification had more prominence, but this makes
the most sense when reading the whitepaper.

## Providing a Security Model

This section builds on the operating system concepts and adds the security
functionality via capabilities.

### Taking Advantage of the Operating System

This section will outline how we can use the operating system to allow, deny,
and audit anything we like. This will not tackle permission/authorisation directly, but simply shows that whatever permission system we choose can use the operating system to disallow certain action etc.

### Implementing a Capbility Based Security Model

This section will outline the capability model we have designed, and how it uses all of the about material to enforce its model.

#### Capabilities and Their Advantages

Not just what capabilities are, but why we have chosen them over the
alternatives.

#### The Beaker Capability Model

**NB:** What's described here is the simplest capability model we could build.
From here we should expand it to make it more complete and featureful.

One of the goals that would improve security and audability of a system is that
and external party or higher level "system designer" might what some control
over what the various conracts in th system can do. This would allow them to
compartmentalise areas of code and ensure that code only has the priveleges it
requires, focussing attention on more critical high-risk code. Even if another
member of the organisation updates a contract under his or her control, the
system designer should be able to limit the potential damage of an error or
malign action by sandboxing that contract.

This can be done by giving each contract (a *procedure* in Beaker parlance) a
set of capabilities, outside of which it cannot act. Even if the procedure is
updated, unless the developer also has the right to increase its capabilities,
system designers and auditors have some guarantees that it will not do something
harmful to parts of the system outside its purview.

The simplest model is simply to give every procedure a list of permitted
actions. Procedure creation is in two steps:

- Creation/Update - Where the contract bytecode is uploaded to the kernel.
- Permission assignation - Where somebody with the appropriate authorisation
  sets the capabilities of the procedure.

**TODO:** Include a diagram of this.

It is critical to note that the capability system proposed here does not attempt
to deal at all with *"users"*. If a particular system hs users (which is to be
expected) it is left to the creators of that system to dictate how that is
organised and implemented. By default, Beaker routes all external transactions
through a (modifiable) procedure which acts as a form of "gatekeeper". It is
within this procedure that decisions about what each user can do are made.

When a procedure is created, it has zero capabilities available to it in its
list. If, for example, it needs to modify the storage value at `0x7`, it will
need to be provided with that permission by a separate permission assignation.
In this workflow, the procedure is deployed by a developer, and the permissions
are assigned by the system designer once he approves this. The workflow around
how permissions are requested and designed are left to the system creators.

**SIDENOTE:** Perhaps we allow a procedure to be run on every syscall what does
whichever system checks the designers deem appropriate although the large number
of procedure calls make this very expensive. It is always possible to provide
system creators with such hooks.

In this situation it would be important to give the system designer powerful
design tools.

**What can't this do?** Something like storage locations or procedure ids can't be
chosen dynamically, as delegation of capabilities does not occur.

Everything is statically determined by the system designer (although permissions
may be changed at any point).

This design has advantages over more dynamic, flexible, capability systems, as
it explains, in a very static and assessable manner:

1. Where do permissions come from?
2. How are they set?
3. How are they enforced?

## Using Beaker to Create a More Secure System

Some information on "usage characteristics". How do you actually use this
properly. This should be high level and not necessarily include code examples or
the like.

## Conclusion

Summarise what we have outlined above and why it is useful.
