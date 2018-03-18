<template>
  <div class="org">
        <b-btn v-b-modal.modalCreateProc size="sm"> Create New Procedure </b-btn>
        <b-table :items="procedures"></b-table>
        <b-modal id="modalCreateProc" ref="modal" title="Create Procedure" @ok="createProcedure">
            <form>
                {{ status }}
                <b-form-textarea id="textarea1"
                            v-model="procedure.code"
                            placeholder="Enter new Procedure Code"
                            :rows="3"
                            :max-rows="100">
                </b-form-textarea>
                <b-form-input v-model="procedure.name" placeholder="Enter new Procedure Name"></b-form-input>
            </form>
        </b-modal>
      
  </div>
</template>

<script>
export default {
  name: "ProcedureView",
  data() {
    return {
      procedures: [],
      procedure: {
        name: "",
        code: ""
      },
      status: "off"
    };
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
    async createProcedure() {
      const instance = this.kernel.instance;
      const web3 = this.$web3();
      const account = this.$accounts()[1];

      try {
        this.status = "running";
        window.Instance = instance;
        let name = web3.utils.toHex(this.procedure.name);
        let code = web3.utils.toHex(this.procedure.code);

        await instance.methods.createProcedure(name, code).send({
          from: account,
          gas: this.$MIN_GAS(),
          gasPrice: this.$MIN_GAS_PRICE()
        });
      } catch (e) {
        console.error(e);
        this.status = "error";
        return;
      }
      this.status = "finished";
      this.getData();

    },
    async getData() {
      let web3 = this.$web3();
      const instance = this.kernel.instance;
      
      const raw = await instance.methods.listProcedures().call();
      console.log(raw)
      const procedures = raw
        .map(web3.utils.toAscii)
        .map(s => s.replace(/\0.*$/, ""))
        .map(name => ({ name }))
     
     for (const i in procedures) {
         console.log(i)
        const address = await instance.methods.getProcedure(web3.utils.toHex(procedures[i].name)).call();
        procedures[i].address = address;
     }
      
      this.procedures = procedures;
    }
  }
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>

</style>
