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

# Proposition B

Under this proposition, there are only 2 relevant permissions for users
(**TODO:** updating):

- Create a procedure.
- Allocate permissions to that procedure.

In effect only one procedure has the ability to create and allocate permissions, and within that procedure is the information required to approve or reject transactions from various addresses. Initially this procedure will only accept transactions from the creator of the kernel. It is, of course, possible for this user to then update this procedure or create a new one.

There are 3 default procedures:

- #1 is the reception procedure, and is always used to process incoming
  transactions
- #2 is the procedure creation procedure.
- #3 is the procedure update procedure.
- #4 is the permission allocation procedure.
The creator of the kernel can send a contract creation request to this kernel.
The kernel will process this transaction using the reception procedure. The
format of this message will state that it wishes to call the contract creation
procedure (procedure #2) to create a procedure. The reception procedure will
determine that whether it is permissible for the account sending the transaction
to do this.

By default the reception procedure simply allows the creator of the kernel to do
whatever he wishes, including replacing the reception procedure. The first thing
the creator wishes to do is to update the reception procedure to allow our
developer to create procedure (not to assign permissions). In order to do this
he creates a new reception procedure. Our new reception procedure will look like
this:

```c
if (sender == creator) {
    allow_all();
} else if (sender == developer) {
    allow_create();
} else {
    allow_none();
}
```

This is done via a call to *update* a particular procedure. This is done by
calling the proecedure update procedure, remembering that the only action
available to external parties is to call a particular procedure.

First the kernel receives the transaction. It then passes it directly to the
reception contract. This contract determines that is is a call to the "procedure
update" procedure, and that the sender (the creator of the kernel instance) has
those permissions. This procedure update procedure is then called with the
bytecode and procedure id that the sender want to replace. The procedure is then
replaced (**TODO:** Need to consider contract destruction on update). After the
update of procedure #1 is complete, the developer now has permission to create
procedures (although, interestingly, not update them).

Once the developer has written his procedure, he compiles it to bytecode. The
resulting code `contract.bin` can then be deployed to the kernel instance using
`beaker deploy`. However, as this procedure has no permissions associated with
it, it cannot perform any state changing actions, certainly not to the member
list. Under the hood, `beaker deploy` simply sends a transaction to the kernel
asking to call procedure \#2 (procedure creation) with the corresponding
bytecode. The updated reception procedure will allow this.

In order to give this procedure any permissions, the permissions must be applied
by someone with the appropriate authority (or maybe even by member vote). For
example, a system administrator, knowing the storage addresses of the member
list, might give this procedure permission to modify the storage at the storage
address of the member list.

## Seeing the Relationships between Procedures and Resources

We want to list our procedures and determine what resources they have permission
to (e.g. storage locations). After uploading his procedure and having the
correct permissions applied, the developer sees the following:

```
# List the capabilities of procedure #2
> beaker list-caps 2
Procedure #2
- WriteStorage 0x8000-0x9000 (0x1000 or 4096 bytes)
- ReadStorage 0x8000-0x9000 (0x1000 or 4096 bytes)
```

This tells us that procedure 2 has write access to 4096 bytes of storage from
0x8000 to 0x9000, and *only* has write access to those bytes, and no others. It
also has read access to those same bytes.
