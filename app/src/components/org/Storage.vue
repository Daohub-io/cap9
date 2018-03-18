<template>
  <div class="org">
      Storage View
  </div>
</template>

<script>
export default {
  name: "StorageView",
  data() {
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
      let kernels = this.$kernels();
      let web3 = this.$web3();
    }
  }
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>

</style>
