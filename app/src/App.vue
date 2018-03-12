<template>
  <div id="app">
    <router-view/>
  </div>
</template>

<script>
import Contracts from 'Contracts'
import Web3 from 'web3'

export default {
  name: 'App'
}

</script>

<style>
* {
  box-sizing: border-box;  
}

html, body, #app {
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 0;
}

#app {
  font-family: 'Avenir', Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

</style>
