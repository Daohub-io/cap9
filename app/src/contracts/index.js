
import Web3 from 'web3'

import KernelABI from './abi/Kernel.json';

const MIN_GAS = 4712388;
const MIN_GAS_PRICE = 100000000000;

// Web3 Vue Plugin
// See: https://vuejs.org/v2/guide/plugins.html
export default {

    accounts: [],

    install(Vue) {

        // Setup Provider, Default with Localhost
        Vue.web3 = new Web3(Web3.givenProvider || "http://localhost:8545"),
        Vue.accounts = [];
        Vue.kernels = new Map();
        
        Vue.prototype.$web3 = () => Vue.web3;
        Vue.prototype.$accounts = () => Vue.accounts;
        Vue.prototype.$kernels = () => Vue.kernels;

        Vue.prototype.$MIN_GAS = () => MIN_GAS;
        Vue.prototype.$MIN_GAS_PRICE = () => MIN_GAS_PRICE;


        Vue.prototype.$connect = async function ({address = "http://localhost:8545"}) {
            Vue.web3.setProvider(address);
            Vue.accounts = await Vue.web3.eth.getAccounts();
        }
    
        Vue.prototype.$createKernel = async function({name, account = false }) {
            if (!account) account = Vue.accounts[0];

            const Kernel = new Vue.web3.eth.Contract([KernelABI])
            let instance = await Kernel.deploy({ data: KernelABI.bytecode }).send({ from: account, gas: MIN_GAS, gasPrice: MIN_GAS_PRICE })
            instance.options.jsonInterface = KernelABI.abi;
            // Add Kernel to List
            Vue.kernels.set(instance.options.address, {name, instance});
            return instance;

        }
    
    }
}



