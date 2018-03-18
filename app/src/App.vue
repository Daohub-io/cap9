<template>
  <div id="app">
    <b-navbar variant="dark" type="dark">
      <b-navbar-brand>Daolab</b-navbar-brand>
      <b-navbar-nav>
        <b-nav-form>
          <b-btn v-b-modal.modalConnection size="sm" :variant="connected ? 'success': 'danger'">{{ connected ? 'Connected': 'No Connection'}}</b-btn>
          <b-modal id="modalConnection" ref="modal" title="Set Connection" @ok="handleOk">
            <form @submit.stop.prevent="handleOk">
              <p>Network Id: {{ network.id }}</p>
              <p>Type: {{ network.type }} </p>
              <b-form-input type="text" placeholder="Enter Node Address" v-model="address"></b-form-input>
            </form>
          </b-modal>
        </b-nav-form>
      </b-navbar-nav>
    </b-navbar>
    <div class="lower-bound">
      <b-nav vertical class="side-nav">
        <b-nav-item to="/org/create">Create</b-nav-item>
        <b-nav-item to="/org/list">List</b-nav-item>
      </b-nav>
      <div class="content">
        <router-view/>
      </div>
    </div>
  </div>
</template>

<script>
import Contracts from "Contracts";
import Web3 from "web3";

// Import Bootstrap
import "bootstrap/dist/css/bootstrap.css";
import "bootstrap-vue/dist/bootstrap-vue.css";

export default {
  name: "App",
  data() {
    return {
      connected: null,
      address: "http://localhost:8545",
      network: {
        id: '',
        type: ''
      }
    };
  },
  created() {
    this.connect();
  },
  methods: {
    async handleOk() {
      await this.connect();
      this.$refs.modal.hide();
    },
    async connect() {
      try {
        await this.$connect({address: this.address})

        const web3 = this.$web3();        
        this.network.id = await web3.eth.net.getId()
        this.network.type = await web3.eth.net.getNetworkType()

      } catch (e) {
        console.error(e, `Invalid Connection`);
        this.connected = false;
        this.network.id = false;
        this.network.type = false;
        return;
      }
      this.connected = true;
    }
  }
};
</script>

<style>
* {
  box-sizing: border-box;
}

html,
body,
#app {
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 0;
}

#app {
  font-family: "Avenir", Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  display: flex;
  flex-direction: column;
}

.lower-bound {
  flex-grow: 1;
  display: flex;
  flex-direction: row;
}

ul.side-nav {
  flex-basis: 12rem;
  background-color: #999;
  }

  ul.side-nav li a {
    border-bottom: 1px solid #888;
    color: #fff;
    padding: 0.7rem 0.5rem;
    width: 100%;
    height: 100%;
  }
  ul.side-nav li a:hover {
    background-color: #333;
  }
.content {
  flex-grow: 1;
}

</style>
