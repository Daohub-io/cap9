# Capabilities


|Index| Kernel Object   | Capability Type | Description                                                                                   |
|---|---------------------|-----------------|---------------------------------------------------------------------------------------------|
|1  | Procedure Table     | Push Cap        | Copy and push a capability from parent's capability list to another procedure with given id |
|2  |                     | Delete Cap      | Delete a capability from a procedure's capability list with given id and cap id             |
|3  |                     | Call            | Call procedure by given id and arguments.                                                   |
|4  |                     | Register        | Register a new procedure with given identifier and capabilities                             |
|5  |                     | Delete          | Delete procedure by identifier.                                                             |
|6  |                     | Entry           | Set the procedure with given identifier as the entry procedure.                             |
|7  | Storage             | Read            | Read from the memory by the given address.                                                  |
|8  |                     | Write           | Write to the memory by the given address.                                                   |
|9  | Log                 | Write           | Append log record with given topics.                                                        |
|10 | Gas                 | Send            | Send Gas to address                                                                         |

Capabilites are split into three categories based on what objects they access: the Procedure Table, Storage, Logs, and Gas.

### Procedure Table
The Procedure Table stores the capability list for each procedure and keeps track of the entry procedure within Kernel Storage.

### Storage
User Storage is storage accessible outside of the Kernel Storage.

### Log
User Logs are Logs accessible to Procedures to Emit

### Gas
Gas can be read and sent

<!-- ### System Call Format -->

<!-- 
The procedure call capability is simply a list of 32-byte values (the values are
simply concatenated without a length value), which are then converted to the
procedure keys which are permitted. If the are no values, the capability allows
the procedure to call any procedure. -->