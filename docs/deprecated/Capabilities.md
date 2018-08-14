# Capabilities

**NB:** This was moved from the github wiki to here.

The beaker exokernel follows a capability-based access-control model. Access
control governs all procedures; in order to perform a system call a procedure
must invoke a capability in its possession that has sufficient access rights for
the requested resource. With this, the system can be configured to isolate
procedures from each other, and also to enable authorised, controlled
communication between procedures by selectively granting specific communication
capabilities. This enables a high degree of assurance, as only those operations
explicitly authorized by capability possession is permitted. A capability is an
unforgeable token that references a specific kernel resource (such as a table)
and carries access rights that control what methods may be invoked.
Conceptually,  a capability resides in a procedures capability space. An address
in this space refers to a slot which may or may not contain a capability. An
application may refer to a capability \- to request a kernel service, for
example \- using the address of the slot holding that capability. This means
Beaker uses a segregated capability model, where capabilities are managed by the
kernel.

Capabilities can be copied and moved within capability spaces. This allows the
creation of procedures with specific access rights, delegation of authority to
another procedure and passing capabilities to a newly created procedure.
Furthermore, capabilities can be minted to create a derived capability with a
subset of the rights of the original capability (never with more rights).

Capabilities can also be invoked to withdraw authority. Revocation recursively
removes any capabilities that have been derived from the original capability
being revoked.

## Storage Capabilities

## Memory Capabilities

## Event Capabilities
