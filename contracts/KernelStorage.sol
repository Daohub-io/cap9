pragma solidity ^0.4.17;

contract KernelStorage {
    event KernelLog(string message);

    // ** KERNEL STORAGE API **
    // These functions operate on the kernel storage. These functions can be
    // considered the kernel storage API, all storage reads and writes should
    // come through this API. Each storage item/location has 3 functions, _set*,
    // _get*, and _getPointer*. The _get and _set functions work directly on
    // the value, while _getPointer returns the storage location. _get and _set
    // rely on _getPointer in such a way that storage locations are defined via
    // the _getPointer functions.

    // _getPointer* functions. These are all defined together first.

    // Returns the storage key that holds the entry procedure name.
    function _getPointerEntryProcedure() pure internal returns (uint256) {
        return 0xffffff0400000000000000000000000000000000000000000000000000000000;
    }

    // Returns the storage key that holds the current procedure name.
    function _getPointerCurrentProcedure() pure internal returns (uint256) {
        return 0xffffff0300000000000000000000000000000000000000000000000000000000;
    }

    // Returns the storage key that holds the kernel address.
    function _getPointerKernelAddress() pure internal returns (uint256) {
        return 0xffffff0200000000000000000000000000000000000000000000000000000000;
    }

    // Return the storage key that holds the number of procedures in the list.
    function _getPointerProcedureTableLength() internal pure returns (uint256) {
        bytes5 directory = bytes5(0xffffffff01);
        return uint256(bytes32(directory));
    }

    // Returns the storage key that holds the procedure data of procedure #idx
    // in the procedure list. idx starts at 1.
    function _getPointerProcHeapByIndex(uint192 idx) internal pure returns (uint256) {
        // TODO: annoying error condition, can we avoid it?
        if (idx == 0) {
            revert("0 is not a valid key index");
        }
        bytes5 directory = bytes5(0xffffffff01);
        return uint256(bytes32(directory)) | (uint256(idx) << 24);
    }

    // Returns the storage key that holds the procedure data with the given
    // procedure name.
    function _getPointerProcHeapByName(uint192 name) internal pure returns (uint256) {
        bytes5 directory = bytes5(0xffffffff00);
        return uint256(bytes32(directory)) | (uint256(name) << 24);
    }
    // The storage key that holds the Procedure Index of a procedure with the
    // given procedure name.
    function _getPointerProcedureIndexOnHeap(uint192 name) internal pure returns (uint256) {
        uint256 pPointer = _getPointerProcHeapByName(name);
        // The procedure index is stored at position 1
        return (pPointer+1);
    }

    // The storage key that holds the Ethereum Address of the code of a
    // procedure with the given procedure name.
    function _getPointerProcedureAddress(uint192 name) internal pure returns (uint256) {
        uint256 pPointer = _getPointerProcHeapByName(name);
        // The procedure index is stored at position 1
        return (pPointer+1);
    }


    // The storage get and set functions

    function _get(uint256 pointer) internal view returns (uint256 val) {
        assembly {
            // Load Value
            val := sload(pointer)
        }
    }

    function _set(uint256 pointer, uint256 value) internal {
        assembly {
            sstore(pointer, value)
        }
    }


    // _get and _set functions

    function _getEntryProcedure() view internal returns (uint192 val) {
        uint256 storageKey = _getPointerEntryProcedure();
        assembly {
            val := sload(storageKey)
        }
        return val;
    }

    function _setEntryProcedureRaw(uint192 procedureKey) internal {
        uint256 storageKey = _getPointerEntryProcedure();
        assembly {
            sstore(storageKey,procedureKey)
        }
    }

    function _getCurrentProcedure() view internal returns (uint192 val) {
        uint256 storageKey = _getPointerCurrentProcedure();
        assembly {
            val := sload(storageKey)
        }
        return val;
    }

    function _setCurrentProcedure(uint192 procedureKey) internal {
        uint256 storageKey = _getPointerCurrentProcedure();
        assembly {
            sstore(storageKey,procedureKey)
        }
    }

    function _getKernelAddress() view internal returns (uint192 val) {
        uint256 storageKey = _getPointerKernelAddress();
        assembly {
            val := sload(storageKey)
        }
        return val;
    }

    function _setKernelAddress(address theAddress) internal {
        uint256 storageKey = _getPointerKernelAddress();
        assembly {
            sstore(storageKey, theAddress)
        }
    }

    // _get and _set functions which use the _getPointer functions above.

    // Given a Procedure Name, return it's index in the Procedure List (i.e. its
    // Procedure Index). If the procedure is not in the list it will return a
    // Procedure Index of zero. Zero is not a valid Procedure Index.
    function _getProcedureIndex(uint192 name) internal view returns (uint192) {
        uint256 procedureIndexPointer = _getPointerProcedureIndexOnHeap(name);
        return uint192(_get(procedureIndexPointer));
    }
}
