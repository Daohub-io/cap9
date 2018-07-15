# Kernel Objects

## CList
To manage and control procedure capabilities kernel has CList. It is access-control list which stores the list of capabilities for each procedure.

## Procedure List
List for procedure creation and deletion. 
//FIXME: The question is do we really need separate list with procedures.
Looks like that CList include this list and there is no sense when procedure exists but don't have any capabilities.

## Events 
Logging

## Storage
Merkle tree
Addresses started with zero byte are reserved for system stuff (clist ...).
Each procedure has it's own storage space.
