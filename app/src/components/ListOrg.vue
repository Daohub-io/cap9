<template>
  <div class="list">
    <b-container>
      <b-table striped hover :items="organizations">
        <template slot="name" slot-scope="data">
          <b-btn size="sm" :to="'/org/'+data.item.address">{{data.item.name}}</b-btn>
        </template>
      </b-table>
    </b-container>
  </div>
</template>

<script>

export default {
  name: "ListOrg",
  data() {
    return {};
  },
  computed: {
    // See http://web3js.readthedocs.io/en/1.0/web3-eth-contract.html#properties
    organizations() {
      let kernels = this.$kernels();
      let web3 = this.$web3();

      return Array.from(kernels).map(([key, {name}]) => {
          return {
              name,
              address: key,
              balance: 0
          };
      });
    }
  }
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>

</style>
