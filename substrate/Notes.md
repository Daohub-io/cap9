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
