# Cap9 SRML Implementation Notes

The environment around Substrate and contracts is still in a lot of flux, so
these notes are simply information that needs to be recorded to guide what is
being done in this directory. The notes don't follow a particular structure but
are all important.

## Testing and Output

A call to a contract does not return any data. Therefore we can't return
information as part of tests. One of the workarounds is to use the `ext_println`
function. This function is only provided when Substrate is run with the `--dev`
option. Unfortunately it simply prints to the stdout of the substrate node
(possibly multiple times) and is therefore not a good option for automated
tests. Looking at the runtime code for the contracts module it seems like it
should be possible, but perhaps that is still a work in progress. There is an
`rpc` subdirectory in the contracts module directory in the master branch, but
it is not in the current snapshot we are using (for compatiblity). This of
course means it's not in much documentation or guidance (tutorials and the like)
which have an even longer lag than compatible builds.

The next option I looked at was performing all of the testing logic within
contracts and if the test fails just crash the contract (e.g. by intentionally
hitting an `unreachable` instruction). The problem with this is that the way
calls work is that an extrinsic (i.e. the call transaction) is submitted and
then the client waits for an event that says the call has been executed.
Currently we don't get any feedback on the success or failure of a call.

A nice way to test would be to test the success of contracts by reading directly
from the contract storage, however, finding the right key is currently
difficult. This is put on hold until there is progress in other areas.

## Calling and Storage

A key part of previous cap9 implementations is that a single contract holds the
storage and each procedure is delegated access to that storage according to the
capability list. In the case of the current substrate contracts, that may no
longer be the case. Unlike Ethereum, Substrate contracts do not appear to have a
concept such as `DELEGATECALL`. `DELEGATECALL` allowed us to delegate to
something like "subcontracts" and let them modify our storage. In Substrate,
with each contract only modifying its own storage, we can't do this. The closest
we can come is to call to other contracts, but allow them to make "system calls"
back to our main contract. To ensure that contracts don't suicide themselves, we
would need to make sure each contract is only called by its "owning" contract.

All of this could be done at the contract code level (as was the case with
Ethereum), and does not actually need runtime support. One advantage is that we
don't need to verify code anymore as it can't modify our storage, however, it
means that we can't store capabilities per-contract, as we are no longer in
control of them. In fact, the earlier concept of using the runtime to store
contract capabilities is useless in this case.

One idea is to have the idea of a "main" contract and "sub" contracts enforced
by the runtime. This would require a lot more work as it means modify the way
contracts are stored and executed, which is far from trivial code.

## Design of Code Isolation

The current implementation on Substrate embeds a primitive capability listed
(associated with each contract) into the runtime. The next question is "how do
we use these capabilities?". As mentioned above, there is no trivial path. To
implement the original design (as on Ethereum) we would need to simply implement
the kernel structure *out* of the runtime (i.e. in contract code). However, this
would also require new opcodes to match those of Ethereum.

Alternatively, we could keep as much implemented in the runtime as possible. The
advantage for this is that is a single standard for the chain, and it can expose
a nice clean interface, just like regular contract functions (i.e. `ext_*`
functions). The problem here is that the old model doesn't fit in the same way.

For all these reasons it is necessary to step through the design again an make
sure every design decision is the right one. It seems that the key thing we need
to keep in mind is that the purpose of this system is:

> to isolate chunks of code and determine categorically (and statically) what
> they are and are not capable of.

Currently the only *unit of isolation* available to Substrate contracts is a
contract. This means that to isolate units of code we must divide them into
separate contracts. If a contract was made up of multiple WASM modules this
would be different. A single WASM module is a natural place to apply
capabilities, but for now the assumption is that each WASM module is a separate
contract.

Next we must determine where our resources are. If our resources can be split
and restricted to a single piece of code as an owner, then we can isolate
resources (such as storage) behind individual contracts. But how do we manage
capabilities then? What is the flow of a transaction and where do the
capabilities come in?

One way would be to isolate resources behind contracts and only allow certain
addresses to call these contracts. This would be difficult to manage as it would
be necessary to update and manage many disparate lists of addresses, and the
mechanism would need to be stored in each contract.

Without secret keys, we need some central storage mechanism to determine what
the capabilities for each unit of isolation are (i.e. our kernel space).
