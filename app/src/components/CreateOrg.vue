<template>
  <div class="hello">
    <b-container>
      <form class="basic" >
        <h2>Create an Organization</h2>
        <input v-model="Org.name" type="text" placeholder=" Name">
        <input type="button" value="Create" v-on:click="createOrg">
        <p v-if="done"> Created </p>
      </form>
    </b-container>
  </div>
</template>

<script>

export default {
  name: "CreateOrg",
  data() {
    return {
      Org: { name: "" },
      done: false,
    };
  },
  methods: {
    async createOrg() {
      let account = this.$accounts()[0];
      try {
        let instance = await this.$createKernel({name: this.Org.name, account });
        // Update Frontend
        let name = this.Org.name;
        this.Org.name = "";
        this.done = true;  

      } catch (e) {
        console.error(e)
      }
    }
  }
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>

</style>
