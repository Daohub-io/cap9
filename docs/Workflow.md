# Workflow

This document covers the workflow that a developer will likely experience using
beaker.

## Setting Up a New Kernel Instance

The first thing they will want to do is set up a new kernel instance. A kernel
instance is and example of a kernel running on an Ethereum blockchain, available
to receive transactions. It uses the BeakerOS library contract to perform most
of its functions.

We'll suppose there is a CLI program `beaker` for managing projects, so we will
use that to demonstrate these concepts. The first thing they will want to do is
start a new project based on a template.

```bash
beaker new
```

This will present a list of templates for the developer to choose from. Let's
assume the developer chooses the default template. This will create the default
"reception procedure" in a new directory. There is no kernel file, as the kernel
itself is simply a standard instance of the kernel. At this point the developer
might want to modify the reception procedure, but this is unlikely except in
particular circumstances. There is also a "create procedure" procedure file. The
developer is also unlikely to want to change this. They do, however, want to
create a ballot procedure, so that members of this organisation can participate
in ballots. However, this organisation has no members list! The developer must
create that first.

For the developer's case, each user is associated with a particular Ethereum
address, as is common practice on Ethereum. This is a living breathing
organisation, so members will need to be added and removed. It goes without
saying that this must be secure and reliable. The first thing to do is to set
aside a piece of storage for the members list, and make sure only the
appropriate procedures and actions can modify it. The developer decides that
this organisation will have no more that 256 members, so allocates a certain
storage space for this (**TODO:** maybe `beaker` can help with the naming and
allocation of this).

Now that the developer has allocated the space, he can then work on the
procedure for adding members. For this we want to add a procedure to our kernel that is able to modify the space we have set aside for this member list.

--- The developer writes his code

**TODO:** This assumes fully dynamic capabilities derived from the reception
procedure.

After he has written the code, he needs to give this procedure access to the
member list. There are two ways he could do this: dynamically and statically. In
the dynamic case, the the procedure would need to be called by a procedure which
has the capability to modify this storage location. As there currently aren't
any procedures that have that capability, we need to assign is statically, which
is equivalent to saying we need to get the kernel to pass it the capabilities.

So how do we do that? Who has the authority? Is it the reception procedure that
allocates capabilities? Does that make all capabilities dynamic. If we don't
limit the capabilities of users and developers in some way, privilege escalation
could become quite a risk.

Here's a design that requires two assertions:

1. The reception procedure has all capabilities.
2. The reception procedure allocates these capabilities to users.

This answers the questions above. Once the reception procedure has figured out
who the user is, it rexecutes itself with only the capabilities of that user.
This then begs the question, what capabilities do we have? How do we find this
out if the capabilities themselves are anonymous. The reception procedure would
have to do some translation from the user into capabilities.
