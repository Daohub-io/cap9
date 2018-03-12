<template>
  <div class="hello">
    <nav>Daolab</nav>
<<<<<<< HEAD
    <main>
      
    </main>
=======
    <main>{{ msg }}</main>
>>>>>>> 50814d4777717b31d4401120107f7b47deee0d43
  </div>
</template>

<script>
export default {
  name: 'HelloWorld',
  data () {
    return {
      msg: 'Welcome to Your Vue.js App'
    }
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
nav, main {
  width: 100%;
}

nav {
  height:2rem;
  background-color: #333;
  color: #fff;
  padding: 2rem;
}

main {
  height: calc(100%-2rem);
}
</style>
