<template>
  <div class="storage">
      <b-container>
        <b-row >
            <b-col cols="2">
                <h5>Storage View</h5>
            </b-col>
        </b-row>
      </b-container>
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
.storage h5 {
    padding: 2rem 0 0;
} 
</style>
