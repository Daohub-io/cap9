# Capabilities

## Procedure Call Capability

Allows a procedure to call another procedure.

### Format

The procedure call capability is simply a list of 32-byte values (the values are
simply concatenated without a length value), which are then converted to the
procedure keys which are permitted. If the are no values, the capability allows
the procedure to call any procedure.
