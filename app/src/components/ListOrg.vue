<template>
  <div class="list">
      <b-table striped hover :items="organizations"></b-table>
  </div>
</template>

<script>
import { KernelABI } from "Contracts";
import Web3 from "web3";

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

      return kernels.map(([name, org]) => {
          return {
              name,
              address: org.options.address,
              balance: 0
          };
      })
    }
  }
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>

</style>
