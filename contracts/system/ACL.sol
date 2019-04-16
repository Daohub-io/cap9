pragma solidity ^0.4.17;

import "../BeakerContract.sol";

contract ACL is BeakerContract {

    struct WriteCap {
        uint256 start;
        uint256 len;
    }
 
    uint8 constant ACCOUNT_MAPPING = 0;
    uint8 constant ACCOUNT_ARRAY = 1;
    uint8 constant GROUP_MAPPING = 2;
    uint8 constant GROUP_ARRAY = 3;

    struct AccountMapVal {
        uint8 accountIndex;
        bytes24 procId;
    }

    struct AccountArrVal {
        address accountId;
    }

    struct GroupMapVal {
        uint8 groupIndex;
    }

    struct GroupArrVal {
        uint8 accountLen;
        bytes24 procId;
    }

    function AccountMapVal_decode(uint256 _data) internal returns (AccountMapVal) {
        return AccountMapVal(uint8(_data >> 24), bytes24(_data));
    }

    function AccountMapVal_encode(AccountMapVal _acc_map_val) internal returns (uint256 data) {
        return (uint256(_acc_map_val.accountIndex) << 24) | uint256(_acc_map_val.procId);
    }


    function AccountArrVal_decode(uint256 _data) internal returns (AccountArrVal) {
        return AccountArrVal(address(_data));
    }

    function AccountArrVal_encode(AccountArrVal _acc_arr_val) internal returns (uint256 data) {
        return uint256(_acc_arr_val.accountId);
    }


    function GroupMapVal_decode(uint256 _data) internal returns (GroupMapVal) {
        return GroupMapVal(uint8(_data));
    }

    function GroupMapVal_encode(GroupMapVal _group_map_val) internal returns (uint256 data) {
        return (uint256(_group_map_val.groupIndex));
    }


    function GroupArrVal_decode(uint256 _data) internal returns (GroupArrVal) {
        return GroupArrVal(uint8(_data >> 24), bytes24(_data));
    }

    function GroupArrVal_encode(GroupArrVal _group_arr_val) internal returns (uint256 data) {
        return (uint256(_group_arr_val.accountLen) << 24) | uint256(_group_arr_val.procId);
    }

    function _getStoreCapsLen() internal returns (uint256 nCaps) {
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getPointerProcHeapByName(_getCurrentProcedure());
        // How many Write capabilities does the current procedure have?
        return _get(currentProcPointer | (CAP_STORE_WRITE*0x10000));
    }

    function _getStoreCap(uint256 index) internal returns (WriteCap writer) {
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getPointerProcHeapByName(_getCurrentProcedure());

        // A write capability has two values, address and size. Address is at
        // 0x00 and size is at 0x01.
        writer.start = _get(currentProcPointer | (CAP_STORE_WRITE*0x10000) | (index + 1)*0x100 | 0x00);
        writer.len = _get(currentProcPointer | (CAP_STORE_WRITE*0x10000) | (index + 1)*0x100 | 0x01);
    }

    function _getGroupByIndex(uint8 _groupIndex) internal returns (bytes24 procId, uint8 accountsLen, uint8 groupIndex) {
        WriteCap memory groupsMap = _getStoreCap(GROUP_MAPPING);
        assert(groupsMap.len > (2 << 20)); // Must be a mapping

        WriteCap memory groupsArray = _getStoreCap(GROUP_ARRAY);
        assert(groupsArray.len >= 256);

        // Get Current Group Data
        GroupArrVal memory current_group_arr = GroupArrVal_decode(read(groupsArray.start + 1 + uint256(_groupIndex)));
        GroupMapVal memory current_group_map = GroupMapVal_decode(read(groupsMap.start + uint256(current_group_arr.procId)));

        procId = current_group_arr.procId;
        accountsLen = current_group_arr.accountLen;
        groupIndex = current_group_map.groupIndex;
    } 

    function _createGroup(bytes24 _procId) internal returns (uint8 groupIndex) {
        // Check that parameters are valid
        assert(_procId != 0);

        WriteCap memory groupsMap = _getStoreCap(GROUP_MAPPING);
        assert(groupsMap.len >= (2 << 20)); // Must be a mapping

        WriteCap memory groupsArray = _getStoreCap(GROUP_ARRAY);
        assert(groupsArray.len >= 256);

        // Check that Groups Array is not full
        uint256 groups_len = read(groupsArray.start + 0);
        assert(groups_len + 2 < groupsArray.len);

        // Get Current Group Data
        GroupMapVal memory current_group_map = GroupMapVal_decode(read(groupsMap.start + uint256(_procId)));
        GroupArrVal memory current_group_arr = GroupArrVal_decode(read(groupsArray.start + 1 + uint256(current_group_map.groupIndex)));

        // Check that group with the same procId doesn't exist.
        assert(current_group_arr.procId != _procId);

        // Update Group with procId and append to groups array
        current_group_map.groupIndex = uint8(groups_len);
        current_group_arr.procId = _procId;
        current_group_arr.accountLen = 0;
    
        write(GROUP_MAPPING, groupsMap.start + uint256(_procId), GroupMapVal_encode(current_group_map));
        write(GROUP_ARRAY, groupsArray.start + 1 + uint256(groups_len), GroupArrVal_encode(current_group_arr));

        // Update Groups Array Length
        write(GROUP_ARRAY, groupsArray.start + 0, groups_len + 1);

        return uint8(groups_len);
    }

    function _removeGroup(bytes24 _procId) internal returns (uint8 groupIndex) {
        revert("Unimplemented!");
    }

    function _getAccountById(address _accountId) internal returns (address accountId, bytes24 procId, uint8 accountIndex) {
        // Get AccountsArray
        WriteCap memory accountsArray = _getStoreCap(ACCOUNT_ARRAY);
        assert(accountsMap.len > 256);

        // Get AccountsMap
        WriteCap memory accountsMap = _getStoreCap(ACCOUNT_MAPPING);
        assert(accountsMap.len > (2 << 20)); // Must be a mapping

        // Get Current Account Data
        AccountMapVal memory current_account_map = AccountMapVal_decode(read(accountsMap.start + uint256(_accountId)));
        AccountArrVal memory current_account_arr = AccountArrVal_decode(read(accountsArray.start + 1 + uint256(current_account_map.accountIndex)));

        accountId = current_account_arr.accountId;
        procId = current_account_map.procId;
        accountIndex = current_account_map.accountIndex;
    }

    function _getAccountByIndex(uint8 _accountIndex) internal returns (address accountId, bytes24 procId, uint8 accountIndex) {
        // Get AccountsArray
        WriteCap memory accountsArray = _getStoreCap(ACCOUNT_ARRAY);
        assert(accountsMap.len > 256);

        // Get AccountsMap
        WriteCap memory accountsMap = _getStoreCap(ACCOUNT_MAPPING);
        assert(accountsMap.len > (2 << 20)); // Must be a mapping

        // Get Current Account Data
        AccountArrVal memory current_account_arr = AccountArrVal_decode(read(accountsArray.start + 1 + uint256(_accountIndex)));
        AccountMapVal memory current_account_map = AccountMapVal_decode(read(accountsMap.start + uint256(current_account_arr.accountId)));

        accountId = current_account_arr.accountId;
        procId = current_account_map.procId;
        accountIndex = current_account_map.accountIndex;
    }

       /// Get GroupId from Account Address
    function getAccountById(address _accountId) public returns (address accountId, bytes24 procId, uint8 accountIndex) {
        // Get AccountsArray
        WriteCap memory accountsArray = _getStoreCap(ACCOUNT_ARRAY);
        assert(accountsMap.len > 256);

        // Get AccountsMap
        WriteCap memory accountsMap = _getStoreCap(ACCOUNT_MAPPING);
        assert(accountsMap.len > (2 << 20)); // Must be a mapping

        // Get Current Account Data
        AccountMapVal memory current_account_map = AccountMapVal_decode(read(accountsMap.start + uint256(_accountId)));
        AccountArrVal memory current_account_arr = AccountArrVal_decode(read(accountsArray.start + 1 + uint256(current_account_map.accountIndex)));

        accountId = current_account_arr.accountId;
        procId = current_account_map.procId;
        accountIndex = current_account_map.accountIndex;
    }



    function _addAccount(address _accountId, uint8 _groupIndex) internal {
        // Get AccountsArray
        WriteCap memory accountsArray = _getStoreCap(ACCOUNT_ARRAY);
        assert(accountsArray.len + 1 > 256);

        // Get AccountsMap
        WriteCap memory accountsMap = _getStoreCap(ACCOUNT_MAPPING);
        assert(accountsMap.len + 1 > (2 << 20)); // Must be a mapping

        // Get Current Account Data
        AccountMapVal memory current_account_map = AccountMapVal_decode(read(accountsMap.start + uint256(_accountId)));
        AccountArrVal memory current_account_arr = AccountArrVal_decode(read(accountsArray.start + 1 + uint256(current_account_map.accountIndex)));

        // Check that Account does not exist
        assert(current_account_arr.accountId == 0);

        // Check that Accounts Array is not full
        uint256 acc_len = read(accountsArray.start + 0);
        assert(acc_len + 2 < accountsArray.len);

        // Get Group Length + Check Group is not full
        WriteCap memory groupArray = _getStoreCap(GROUP_ARRAY);
        assert(groupArray.len + 1 > 256);

        // Get Group Index
        WriteCap memory groupsMap = _getStoreCap(GROUP_MAPPING);
        assert(groupsMap.len > (2 << 20)); // Must be a mapping

        GroupMapVal memory current_group_map = GroupMapVal_decode(read(groupsMap.start + uint256(current_account_map.procId)));
        GroupArrVal memory current_group = GroupArrVal_decode(read(groupArray.start + 1 + uint256(current_group_map.groupIndex)));

        assert(current_group.accountLen + 1 < 256);

        // Set Index to Current Accounts Length
        uint256 account_map_data = AccountMapVal_encode(AccountMapVal(_groupIndex, current_account_map.procId));
        uint256 account_arr_data = AccountArrVal_encode(AccountArrVal(_accountId));

        write(ACCOUNT_MAPPING, accountsMap.start + uint256(_accountId), account_map_data);
        write(ACCOUNT_ARRAY, accountsArray.start + 1 + acc_len, account_arr_data);

        // Update Group Length
        write(GROUP_ARRAY, groupArray.len + 0, GroupArrVal_encode(GroupArrVal(current_group.accountLen + 1, current_group.procId)));
    }

    function _removeAccount(address _accountId) internal {
        // Get AccountsArray
        WriteCap memory accountsArray = _getStoreCap(ACCOUNT_ARRAY);
        assert(accountsArray.len + 1 > 256);

        // Get AccountsMap
        WriteCap memory accountsMap = _getStoreCap(ACCOUNT_MAPPING);
        assert(accountsMap.len + 1 > (2 << 20)); // Must be a mapping

        // Get Current Account Data
        AccountMapVal memory current_account_map = AccountMapVal_decode(read(accountsMap.start + uint256(_accountId)));
        AccountArrVal memory current_account_arr = AccountArrVal_decode(read(accountsArray.start + 1 + uint256(current_account_map.accountIndex)));

        // Check that Account does exist
        assert(current_account_arr.accountId != 0 && current_account_arr.accountId == _accountId);

        // Check that Accounts Array is not empty
        uint256 acc_len = read(accountsArray.start + 0);
        assert(acc_len > 1);

        // Get Group Length + Check Group is not full
        WriteCap memory groupArray = _getStoreCap(GROUP_ARRAY);
        assert(groupArray.len + 1 > 256);

        WriteCap memory groupsMap = _getStoreCap(GROUP_MAPPING);
        assert(groupsMap.len > (2 << 20)); // Must be a mapping

        // Get Group Index
        GroupMapVal memory current_group_map = GroupMapVal_decode(read(groupsMap.start + uint256(current_account_map.procId)));
        GroupArrVal memory current_group = GroupArrVal_decode(read(groupArray.start + 1 + uint256(current_group_map.groupIndex)));

        assert(current_group.accountLen + 1 < 256);
        assert(current_group.accountLen > 0);

        // Clear Mapping Value
        write(ACCOUNT_MAPPING, accountsMap.start + uint256(_accountId), 0);

        // If Account Index is not Last take Last Account and Move it to the deleted Account Index
        if (current_account_map.accountIndex + 1 != acc_len) {
            // Get Current Last Account Data
            AccountMapVal memory last_account_map = AccountMapVal_decode(read(accountsMap.start + uint256(_accountId)));

            // Update Account Map Index
            last_account_map.accountIndex = current_account_map.accountIndex;
            write(ACCOUNT_MAPPING, accountsMap.start + uint256(_accountId), AccountMapVal_encode(last_account_map));

            // Replace Removed Account Array Value with Last Array Value
            uint256 last_account_arr_raw = read(accountsArray.start + 1 + acc_len);
            write(ACCOUNT_ARRAY, accountsArray.start + 1 + current_account_map.accountIndex, last_account_arr_raw);
        }

        // Delete Last Account and Update Arr Length
        write(ACCOUNT_ARRAY, accountsArray.start + 1 + acc_len, 0);
        write(ACCOUNT_ARRAY, accountsArray.start + 0, acc_len - 1);

        // Update Group Length
        write(GROUP_ARRAY, groupArray.start + current_group_map.groupIndex, GroupArrVal_encode(GroupArrVal(current_group.accountLen - 1, current_group.procId)));
    }
    
    /// Add Account to associate to a group
    /// Only Callable by Group Member
    // function addAccount(address _account, bytes24 _groupId) public {
        
    //     // Check for Valid Parameters
    //     assert(_groupId != 0);
    //     assert(_account != 0);

    //     // Check for Valid Caller
    //     assert(tx.origin != _account);
    //     assert(getAccountGroup(tx.origin) == _groupId);

    //     WriteCap memory accountsArray = _getStoreCap(ACCOUNT_ARRAY);
    //     assert(accountsArray.len + 1 > 256);

    //     // Check that Accounts Array is not full
    //     uint256 acc_len = _get(accountsArray.start + 0);
    //     assert(acc_len + 2 < accountsMap.len);

    //     WriteCap memory accountsMap = _getStoreCap(ACCOUNT_MAPPING);
    //     assert(accountsMap.len > (2 << 20)); // Must be a mapping

    //     WriteCap memory groupArray = _getStoreCap(GROUP_ARRAY);
    //     assert(groupArray.len > 256);

    //     WriteCap memory groupMap = _getStoreCap(GROUP_MAPPING);
    //     assert(groupMap.len > (2 << 20)); // Must be a mapping

    //     // Get AccountIndex
    //     AccountMapVal memory account_d = AccountMapVal_decode(read(accountsMap.start + uint256(_account)));

    //     // Get AccountId
    //     AccountArrVal memory account_darr = AccountArrVal_decode(read(accountsArray.start + 1 + uint256(account_d.accountIndex)));

    //     // Get GroupId
    //     uint256 g_arr_data = read(groupArray.start + 1 + uint256(account_d.groupIndex));
    //     GroupArrVal memory group_darr = GroupArrVal_decode(g_arr_data);

    //     // Get GroupIndex
    //     uint256 g_index_data = read(groupMap.start + uint256(_groupId));
    //     GroupMapVal memory group_dmap = GroupMapVal_decode(g_arr_data);

    //     // If Account already exists
    //     if(account_darr.accountId == _account) {
    //         // If the Proc Id is the same, do nothing, else error.
    //         if (group_darr.procId == _groupId) {
    //             return;
    //         } else {
    //             revert("Account already set by other group");
    //         }
    //     } else {
    //         // If Account is new
    //         uint256 new_account_arr_data = AccountArrVal_encode(AccountArrVal(_account));
    //         uint256 new_account_map_data = AccountMapVal_encode(AccountMapVal(group_dmap.groupIndex, uint8(acc_len) + 1));

    //         write(ACCOUNT_MAPPING, accountsMap.start + uint256(_account), new_account_map_data);
    //         write(ACCOUNT_ARRAY, accountsArray.start + 1 + acc_len + 1, new_account_arr_data);
    //         write(ACCOUNT_ARRAY, accountsArray.start + 0, acc_len + 1);
    //     }
    // }

}