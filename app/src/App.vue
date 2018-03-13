<template>
  <div id="app">
    <b-navbar variant="dark" type="dark">
      <b-navbar-brand>Daolab</b-navbar-brand>
      <b-navbar-nav>
        <b-nav-form>
          <b-btn v-b-modal.modalConnection size="sm" :variant="connected ? 'success': 'danger'">{{ connected ? 'Connected': 'No Connection'}}</b-btn>
          <b-modal id="modalConnection" ref="modal" title="Set Connection" @ok="handleOk">
            <form @submit.stop.prevent="handleOk">
              <b-form-input type="text" placeholder="Enter Node Address" v-model="address"></b-form-input>
            </form>
          </b-modal>
        </b-nav-form>
      </b-navbar-nav>
    </b-navbar>
    <div class="lower-bound">
      <b-nav vertical class="side-nav">
        <b-nav-item>
          <router-link to="/createOrg">Create</router-link>
        </b-nav-item>
        <b-nav-item>
          <router-link to="/listOrg">List</router-link>
        </b-nav-item>
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
      address: "http://localhost:8545"
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
      } catch (e) {
        console.error(e, `Invalid Connection`);
        this.connected = false;
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
.content {
  flex-grow: 1;
}

</style>
