<template>
  <div class="has-text-font-color">
    <div class="container is-max-desktop">
    <!-- <div class="container is-max-desktop has-text-navy"> -->
      <!-- Title -->
      <div class="columns">
        <div class="column">
          <Title/>
        </div>
      </div>
      <!-- Preparation -->
      <div class="columns has-text-weight-bold">
        <div class="column is-size-3">
          1. Preparation
        </div>
      </div>

      <!-- Reference -->
      <div class="columns pl-2 has-text-weight-semibold">
        <div class="column is-size-4">
          (1) Make Reference
        </div>
      </div>
      <!-- New Button -->
      <div class="columns pl-4">
        <div class="column">
          <button
          class="button is-navy is-normal"
          v-if="!reference_state.exist && !reference_generator.opened"
          @click="this.reference_generator.opened = true">
            New
          </button>
        </div>
      </div>
      <!-- Generator Field -->
      <div class="pl-4" v-if="!reference_state.exist && reference_generator.opened">
        <!-- Default Field -->
        <div class="columns">
          <div class="column">
            <block>Input FASTA string</block>
          </div>
        </div>
        <div class="columns" v-if="!reference_generator.input_is_path">
          <div class="column">
            <textarea class="textarea" v-model="reference_generator.fasta_file_bytes"
            placeholder="e.g.&#10;> record_1&#10;AATGTAAACACTCTTGCTCTTGCAAATTCTTCATCTGTACTTATAGCTTTCCAGTTTCTTGACTCTGGATTATAAGAAAGCGTTTTATTACCTTTTGAATCCCAAAACATACATGCAGCATTCATTTTGCCACCAGTTTTTTTCATGCTTGATTCATATATAGCCTTTCTATCAGGAGATACTGTTTCTCCATGCTGCATACA&#10;> record_2&#10;CAATTTTCGATAAGCATCATCATCCCTTTTTCCAGTAGCAAACTCTTTTCTTGCAAGTTCTTTTATTGCTTCGTCAAATTCTTCCTCTGACATCGCTGGTTTATCTCGTTTTGTCATGATAGTATCCCAGTTTGGTTTGGTAAAATTAATGTCCACAGGCTTAAATCTTAATGAGAAATTACCATAATCACATTGTGCTAACGGATCATTTGAACTTGCTTTTTTCCCGAACATC"></textarea>
          </div>
        </div>
        <div class="columns" v-if="reference_generator.input_is_path">
          <div class="column">
            <div class="file has-name">
              <label class="file-label">
                <input
                class="file-input"
                type="file"
                name="resume"
                @change="chooseFastaFile"
                >
                <span class="file-cta">
                  <span class="file-label">
                    Choose a FASTA file..
                  </span>
                </span>
                <span class="file-name">
                  {{ reference_generator.fasta_file_name }}
                </span>
              </label>
            </div>
          </div>
        </div>
        <div class="column">
          <div class="field">
            <input class="switch is-rtl is-navy is-normal"
            id="switchFastaIsFile"
            type="checkbox"
            name="switchFastaIsFile"
            v-model="reference_generator.input_is_path">
            <label for="switchFastaIsFile">Input file instead</label>
          </div>
        </div>
        <div class="column">
          <!-- Advanced options -->
          <div class="field">
            <input class="switch is-rtl is-navy is-normal"
            id="switchAdvancedOption"
            type="checkbox"
            name="switchAdvancedOption"
            v-model="reference_generator.advanced_opened">
            <label for="switchAdvancedOption">Change advanced options</label>
          </div>
        </div>

        <div class="column pl-4" v-if="reference_generator.advanced_opened">
          <!-- Sampling Ratio -->
          <div class="columns">
            <div class="column is-one-quarter">
              Sampling ratio
            </div>
            <div class="column is-one-fifth">
              <div class="field">
                <div class="control">
                  <input class="input" type="text" placeholder="Suffix array sampling ratio"
                  v-model="reference_generator.sampling_ratio">
                </div>
              </div>
            </div>
          </div>
          <!-- Kmer size for lookup table -->
          <div class="columns">
            <div class="column is-one-quarter">
              Kmer size
            </div>
            <div class="column is-one-fifth">
              <div class="field">
                <div class="control">
                  <input class="input" type="text" placeholder="Kmer size for lookup table"
                  v-model="reference_generator.lookup_table_kmer_size">
                </div>
              </div>
            </div>
          </div>
          <!-- BWT Block -->
          <div class="columns">
            <div class="column is-one-quarter">
              BWT block size
            </div>
            <div class="column is-one-fifth">
              <div class="field">
                <div class="control">
                  <label class="radio">
                    <input type="radio" value="64" v-model="reference_generator.bwt_block_size">
                    64
                  </label>
                  <label class="radio">
                    <input type="radio" value="128" v-model="reference_generator.bwt_block_size">
                    128
                  </label>
                </div>
              </div>
            </div>
          </div>
        </div>
        <!-- Generate -->
        <div class="columns">
          <div class="column">
            <button
            class="button is-navy is-normal"
            @click="generateReference">
              Generate
            </button>
          </div>
        </div>
        <div class="columns" v-if="reference_generator.error_exist">
          <div class="column">
            <block>{{ reference_generator.error_msg }}</block>
          </div>
        </div>
      </div>
      <!-- Reference State -->
      <div class="pl-4" v-if="reference_state.exist">
        <div class="columns">
          <div class="column">
            {{ reference_state }}
          </div>
        </div>
        <div class="columns">
          <div class="column">
            <button
            class="button is-navy is-normal"
            @click="resetReference">
              Reset
            </button>
          </div>
        </div>
      </div>

      <!-- Aligner -->
      <div class="columns pl-2 has-text-weight-semibold">
        <div class="column is-size-4">
          (2) Make Aigner
        </div>
      </div>
      <!-- New Button -->
      <div class="columns pl-4">
        <div class="column">
          <button
          class="button is-navy is-normal"
          v-if="!aligner_state.exist && !aligner_generator.opened"
          @click="this.aligner_generator.opened = true">
            New
          </button>
        </div>
      </div>
      <!-- Generator Field -->
      <div class="pl-4" v-if="!aligner_state.exist && aligner_generator.opened">
        <!-- Penalties -->
        <div class="columns">
          <div class="column">
            Penalties
          </div>
        </div>
        <div class="pl-4">
          <div class="columns">
            <div class="column is-one-quarter">
              Mismatch penalty
            </div>
            <div class="column is-one-fifth">
              <div class="field">
                <div class="control">
                  <input class="input" type="text" placeholder="mismatch penalty"
                  v-model="aligner_generator.mismatch_penalty">
                </div>
              </div>
            </div>
          </div>
          <div class="columns">
            <div class="column is-one-quarter">
              Gap-open penalty
            </div>
            <div class="column is-one-fifth">
              <div class="field">
                <div class="control">
                  <input class="input" type="text" placeholder="gap-open penalty"
                  v-model="aligner_generator.gap_open_penalty">
                </div>
              </div>
            </div>
          </div>
          <div class="columns">
            <div class="column is-one-quarter">
              Gap-extend penalty
            </div>
            <div class="column is-one-fifth">
              <div class="field">
                <div class="control">
                  <input class="input" type="text" placeholder="gap-extend penalty"
                  v-model="aligner_generator.gap_extend_penalty">
                </div>
              </div>
            </div>
          </div>
        </div>
        <!-- Cutoff -->
        <div class="columns">
          <div class="column">
            Cutoff
          </div>
        </div>
        <div class="pl-4">
          <div class="columns">
            <div class="column is-one-quarter">
              Minimum aligned length
            </div>
            <div class="column is-one-fifth">
              <div class="field">
                <div class="control">
                  <input class="input" type="text" placeholder="minimum aligned length"
                  v-model="aligner_generator.minimum_aligned_length">
                </div>
              </div>
            </div>
          </div>
          <div class="columns">
            <div class="column is-one-quarter">
              Maximum penalty per length
            </div>
            <div class="column is-one-fifth">
              <div class="field">
                <div class="control">
                  <input class="input" type="text" placeholder="maximum penalty per length"
                  v-model="aligner_generator.maximum_penalty_per_length">
                </div>
              </div>
            </div>
          </div>
        </div>
        <!-- Generate -->
        <div class="columns">
          <div class="column">
            <button
            class="button is-navy is-normal"
            @click="generateAligner">
              Generate
            </button>
          </div>
        </div>
        <div class="columns" v-if="aligner_generator.error_exist">
          <div class="column">
            <block>{{ aligner_generator.error_msg }}</block>
          </div>
        </div>
      </div>
      <!-- Aligner State -->
      <div class="pl-4" v-if="aligner_state.exist">
        <div class="columns">
          <div class="column">
            {{ aligner_state }}
          </div>
        </div>
        <div class="columns">
          <div class="column">
            <button
            class="button is-navy is-normal"
            @click="resetAligner">
              Reset
            </button>
          </div>
        </div>
      </div>

      <!-- Alignment -->
      <div class="columns has-text-weight-bold">
        <div class="column is-size-3">
          2. Alignment 
        </div>
      </div>
      <!-- Select Query -->
      <div class="pl-2">
        <div class="columns has-text-weight-semibold">
          <div class="column is-size-4">
            (1) Input Query
          </div>
        </div>
        <div class="columns pl-2">
          <div class="column">
            <textarea class="textarea" v-model="query_string" placeholder="Input Query String"></textarea>
          </div>
        </div>
      </div>
      <!-- Select Algorithm -->
      <div class="pl-2">
        <div class="columns has-text-weight-semibold">
          <div class="column is-size-4">
            (2) Select Algorithm
          </div>
        </div>
        <div class="columns pl-2">
          <div class="column is-one-fifth">
            <button class="button is-navy is-normal" @click="semiglobalAlign">
              Semi-global
            </button>
          </div>
          <div class="column">
            Returns the alignment containing the end of the sequence.
          </div>
        </div>
        <div class="columns pl-2">
          <div class="column is-one-fifth">
            <button class="button is-navy is-normal" @click="localAlign">
              Local
            </button>
          </div>
          <div class="column">
            Returns the longest alignment.
          </div>
        </div>
      </div>
      <!-- Alignment Result -->
      <hr class="has-background-navy">
      <div class="columns has-text-weight-bold">
        <div class="column is-size-3">
          Alignment Result
        </div>
      </div>
      <div class="pl-2">
        
        <div class="columns pl-2">
          <div class="column">
            {{ result }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import Title from './components/Title.vue';
import { Sigalign } from '../../sigalign-demo-wasm/pkg';

const wasm = import("../../sigalign-demo-wasm/pkg");

const sigalign_promise: Promise<Sigalign> = wasm
  .then((value) => {
    return new value.Sigalign();
  });

interface ReferenceState {
  exist: boolean,
  is_nucleotide: boolean,
  noise_character: string,
  sampling_ratio: number,
  bwt_block_size: number,
  lookup_table_kmer_size: number,
}
interface AlignerState {
  exist: boolean,
  mismatch_penalty: number,
  gap_open_penalty: number,
  gap_extend_penalty: number,
  minimum_aligned_length: number,
  maximum_penalty_per_length: number,
  pattern_size: number,
}

interface ReferenceGenerator {
  opened: boolean,
  input_is_path: boolean,
  fasta_file_bytes: string,
  fasta_file_name: string,
  advanced_opened: boolean,
  sampling_ratio: number,
  bwt_block_size: number,
  lookup_table_kmer_size: number,
  error_exist: boolean,
  error_msg: string,
}
interface AlignerGenerator {
  opened: boolean,
  mismatch_penalty: number,
  gap_open_penalty: number,
  gap_extend_penalty: number,
  minimum_aligned_length: number,
  maximum_penalty_per_length: number,
  error_exist: boolean,
  error_msg: string,
}

export default defineComponent({
  name: 'App',
  components: {
    Title,
  },
  data() {
    return {
      reference_state: {
        exist: false,
        is_nucleotide: false,
        noise_character: "",
        sampling_ratio: 0,
        bwt_block_size: 0,
        lookup_table_kmer_size: 0,
      } as ReferenceState,
      aligner_state: {
        exist: false,
        mismatch_penalty: 0,
        gap_open_penalty: 0,
        gap_extend_penalty: 0,
        minimum_aligned_length: 0,
        maximum_penalty_per_length: 0,
        pattern_size: 0,
      } as AlignerState,
      reference_generator: {
        opened: false,
        input_is_path: false,
        fasta_file_bytes: ">text\nAGCGTTTTATTACCTTTTGAATCCCAAAACATACATGCAGCATTCATTTTGCCACCAGTTTTTTTCATGCTTGATTCATATATAGCCTTTCTATCAGGAGATACTGTTTCTCCATGCTGCATACACAATTTTCGATAAGCATCATCATCCCTTTTTCCAGTAGCAAACTCTTTTCTTGCAAGTTCTTTTATTGCTTCGTCAAATTCTTCCTCTGACATCGCTGGTTTATCTCGTTTTGTCATGATAGTATCCCAGTTTGGTTTGGTAAAATTAATGTCCACAGGCTTAAATCTTAATGAG",
        fasta_file_name: "",
        advanced_opened: false,
        sampling_ratio: 2,
        bwt_block_size: 64,
        lookup_table_kmer_size: 4,
        error_exist: false,
        error_msg: "",
      } as ReferenceGenerator,
      aligner_generator: {
        opened: false,
        mismatch_penalty: 4,
        gap_open_penalty: 6,
        gap_extend_penalty: 2,
        minimum_aligned_length: 100,
        maximum_penalty_per_length: 0.05,
        error_exist: false,
        error_msg: "",
      } as AlignerGenerator,
      query_string: "",
      result: "None",
    }
  },
  methods: {
    generateReference() {
      sigalign_promise.then((sigalign) => {
        let result = sigalign.generate_reference(
          this.reference_generator.fasta_file_bytes,
          this.reference_generator.sampling_ratio,
          this.reference_generator.bwt_block_size,
          this.reference_generator.lookup_table_kmer_size,
          );
        console.log(result);
        if (result == "") {
          this.reference_state = JSON.parse(sigalign.reference);
          this.reference_generator.opened = false;
          this.reference_generator.error_exist = false;
          this.reference_generator.error_msg = "";
        } else {
          this.reference_generator.error_exist = true;
          this.reference_generator.error_msg = result;
        }
      })
    },
    chooseFastaFile(event: InputEvent) {
      console.log(event.data);
      console.log(event.inputType);
    },
    resetReference() {
      sigalign_promise.then((sigalign) => {
        sigalign.reset_reference();
        this.reference_state = JSON.parse(sigalign.reference);
      })
    },
    generateAligner() {
      sigalign_promise.then((sigalign) => {
        let result = sigalign.generate_aligner(
          this.aligner_generator.mismatch_penalty,
          this.aligner_generator.gap_open_penalty,
          this.aligner_generator.gap_extend_penalty,
          this.aligner_generator.minimum_aligned_length,
          this.aligner_generator.maximum_penalty_per_length,
          );
        console.log(result);
        if (result == "") {
          this.aligner_state = JSON.parse(sigalign.aligner);
          this.aligner_generator.opened = false;
          this.aligner_generator.error_exist = false;
          this.aligner_generator.error_msg = "";
        } else {
          this.aligner_generator.error_exist = true;
          this.aligner_generator.error_msg = result;
        }
      })
    },
    resetAligner() {
      sigalign_promise.then((sigalign) => {
        sigalign.reset_aligner();
        this.aligner_state = JSON.parse(sigalign.aligner);
      })
    },
    semiglobalAlign() {
      sigalign_promise.then((sigalign) => {
        let result = sigalign.semi_global_alignment(this.query_string);
        this.result = result;
      })
    },
    localAlign() {
      sigalign_promise.then((sigalign) => {
        let result = sigalign.local_alignment(this.query_string);
        this.result = result;
      })
    },
  },
  mounted() {
    sigalign_promise.then((sigalign) => {
      this.reference_state = JSON.parse(sigalign.reference);
      this.aligner_state = JSON.parse(sigalign.aligner);
    })
  },
})
</script>

<style lang="scss">
@import "./assets/style.scss";
</style>
