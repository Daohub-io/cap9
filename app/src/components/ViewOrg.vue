<template>
  <div class="org">
       <b-nav class="sub_bar">
           <b-nav-item disabled> {{ kernel.name }} </b-nav-item>
            <b-nav-item :to="{name: 'k_storage'}" :active="$route.name == 'k_storage'">Storage</b-nav-item>
            <b-nav-item :to="{name: 'k_procedures'}" :active="$route.name == 'k_procedures'">Procedures</b-nav-item>
        </b-nav>
      <router-view/>
  </div>
</template>

<script>
export default {
  name: "ViewOrg",
  data() {
    let id = this.$route.params.id;

    // See http://web3js.readthedocs.io/en/1.0/web3-eth-contract.html#properties
    return {};
  },
  mounted() {
    this.getData();
  },
  computed: {
    kernel() {
      return this.$kernels().get(this.$route.params.id);
    }
  },
  methods: {
    async getData() {
      let web3 = this.$web3();
    }
  }
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
.sub_bar {
  background-color: #ddd;
}
</style>
