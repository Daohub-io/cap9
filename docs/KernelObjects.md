# Kernel Objects

## Procedure

### Create
Create procedure with given identifier (id), capabilities and root flag.

#### Arguments
* id - identifier of the new procedure
* clist - list of capabilities for the new procedure
* root flag - boolean value whitch indicates that this procedure become the root procedure 

#### Return Value
OK

#### Errors
* id_already_exists - procedure with given id already exists

### Call
Call procedure with given id and with given arguments (?).

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

### Create

### Read

### Write

## Log
### Create
### Write

## Capabilities
### Mint
### Copy
### Delete

## Gas
### Received
### Available
### Send
### Cost

