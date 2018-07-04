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

How do our capabilities work

## Using Beaker to Create a More Secure System

Some information on "usage characteristics". How do you actually use this
properly. This should be high level and not necessarily include code examples or
the like.

## Conclusion

Summarise what we have outlined above and why it is useful.
