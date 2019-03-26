# Alignment and Packing

When laying out some changes (due to our recent discussions) to system calls and
capabilities I've come to come conclusions that we should consider. You may have
considered this similarly, but I thought I would record it to make sure we're on
the same page.

By alignment I mean: if we have two 3-byte values, should we just manipulate
them as two 32-byte words (i.e. 32-byte aligned) or should we put them together
within a single 32-byte word (which is EVM's native word size).

It is important to distinguish between capabilities and system calls, because
the tradeoffs are very different. System calls are never put in storage, they
are simply put in memory, sent to the kernel, and then put on the stack.
Capabilities are stored in storage once and then read many times from storage
directly to the stack.

With those differences in mind, if we look at some gas costs, stack
manipulations might be given numbers between 3 and 5, with memory being the same
but scaling upward as we use more of it. Writing a value to storage is in the
many thousands, but that is amortised so we won't consider it. Reading from
storage is about 200.

The cost of packing multiple values into one word is that you will have to
perform a DUP-PUSH-DIV sequence to get your chosen value. If we consider that
costs about 9, then we see that for anything in storage this is a no-brainer. 9
<< 200. Therefore we can conclude that if we can reasonably pack multiple values
of capabilities into a single storage key, then we should.

The tradeoff for system calls is a little different. If we look only at
CALLDATALOADs, one CALLDATALOAD is cheaper than the PUSH-DIV sequence necessary
to unpack a value. It is also a little easier for consumers of the kernel's API
to simply work in 32-byte values. Therefore, for system calls we can comfortably
throw 32-byte values around without being overly concerned about cost. One
caveat of course is that we have to pay for the transaction transfer (i.e. the
transaction payload). This costs 68 per non-zero byte and 4 per zero-byte. Two
concatenated 3-byte values is 68*3*2=408 plus about 18 for stack manipulation,
which gives 426. Two 32-byte values with no stack manipulation is 4,532.

For SysCalls I think once we have a better idea what the costs/issues are in a
working alpha, we can then optimize syscall costs as a low-hanging fruit. So for
now let's do this:

> For Capability Storage we can do our own custom packing.For System Calls let's
> use standard ABI encoding. Such that we are compatible with abi#encode and
> abi#decode if possible.

Looking at the current spec, the most notable difference would be proc#call
syscall format, we're there's some packing involved, (which I think could be
packed even further since capIndex has a max size of 1 byte). I think though
it's a bug issue for now if we have everything unpacked for now.

I agree pretty much with what you've had and that's loosely the approach I took
too. In fact that's what this issue was supposed to go through. It was only when
typing it out when I calculated that the difference due to transaction data
costs was also worth considering. We already pack the capability type down to
one byte, so I've written that capType and capIndex are packed, as they're
common to each syscall, and indiviual syscall data is as simple as possible.
This is only a slight departure from what we've discussed about. To summarise:

> Capability storage is packed as much as is simple and practical. General
> system call data (capType and capIndex) are packed. System call formats for
> each type of system call are as loosely packed as practical.

If you keep this issue open a while longer I'll put it in a markdown doc
somewhere as a design policy, then close it. That's probably how I should have
approached it.
