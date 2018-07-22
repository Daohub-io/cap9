# Kernel Objects

## Procedure
For each procedure kernel keeps it's identifier, capability list (clist) and root flag.
For making system call procedure provides capability to kernel simply by sending to it the index of the capability from it's clist.
Root flag shows that this procedure is the root procedure - the procedure, which will be called after user's request (kind of entry point).

System calls:
### Create
Create procedure with given identifier (id), capabilities and root flag.
Capabilites are minted from the capabilities of the parent procedure with clist argument.

#### Arguments
* id - identifier of the new procedure
* mlist - array of same size as the clist of parent procedure and contains logic of minting capabilities of the parent procedure to child procedure
* root flag - boolean value whitch indicates that this procedure become the root procedure 

#### Return Value
OK

#### Errors
* id_already_exists - procedure with given id already exists

### Call
Call procedure by given id and with given arguments (?).

#### Arguments
* id - identifier of the procedure to call
* args - list of arguments

#### Return Value
The result of the procedure.

#### Errors
* unknown_procedure - no procedure with given id
* procedure_error - some error happened during procedure execution

### Delete
Delete procedure by id.

#### Arguments
* id - identifier of the procedure

#### Return Value
OK

#### Errors
* unknown_id - procedure with given id doesn't exist

### Root
Set the procedure with given id as the root procedure.

### Arguments
* id - procedure id

### Return Value
OK

### Errors
* unknown_id - procedure with given id doesn't exist

## Storage

### Read
Read from the memory by the given address.

#### Arguments
* addr - memory address

#### Return Value
256 bits stored in the given address

### Write
Write to the memory by the given address. 

#### Arguments
* addr - memory address
* value - 256 bits to store 

#### Return Value
256 bytes stored in the given address

## Log
### Write
Append log record with given topics.
Size of the topics array determines the log opcode, and that's why it has to be less or equal to 4. 

#### Arguments
* topics - array of topics

#### Return Value
OK

#### Errors
* illegal_topics - illegal size of the topics array  

## Gas
### Received
The total amount of gas received from user.

#### Return Value
Amount of gas.
