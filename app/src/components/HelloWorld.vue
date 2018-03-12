<template>
  <div class="hello">
    <nav>Daolab</nav>
    <main>
      <form class="basic" >
        <h2>Create an Organization</h2>
        <input v-model="Org.name" type="text" placeholder=" Name">
        <input type="button" value="Create" v-on:click="createOrg">
        <p> {{Org.name}}</p>
        <p v-if="done"> Created </p>
        <p> Connection: {{ status.value }} </p>
      </form>
    </main>
  </div>
</template>

<script>
import {KernelABI} from "Contracts";
import Web3 from 'web3';

// Setup Provider, Default with Localhost
const web3 = new Web3(Web3.givenProvider || "http://localhost:8545");
const Kernel = new web3.eth.Contract([KernelABI])

export default {
  name: "HelloWorld",
  data() {
    return {
      Org: { name: "" },
      List: [],
      done: false,
      status: {
        value: '',
        connected: true,
        accounts: []
      },
    };
  },
  created() {
    this.connect();
  },
  methods: {
    async createOrg() {
      let account1 = this.status.accounts[0];
      try {
        let instance = await Kernel.deploy({ data: KernelABI.bytecode }).send({ from: account1, gas: 4712388, gasPrice: 100000000000 })

        // Update Frontend
        let name = this.Org.name;
        this.List.push({ name: instance });
        this.Org.name = "";
        this.done = true;  
      } catch (e) {
        console.error(e)
      }
    },
    async connect() {
      try {
        this.status.connected = await web3.eth.isSyncing();
        this.status.accounts = await web3.eth.getAccounts();
      } catch (e) {
        this.status.value = "Invalid Connection"
        return;
      }
      this.status.value = "Connected"

    }
  }
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
nav,
main {
  width: 100%;
}

nav {
  height: 2rem;
  background-color: #333;
  color: #fff;
  padding: 2rem;
}

main {
  height: calc(100%-2rem);
}

form.basic {
  padding: 1rem;
}
</style>
