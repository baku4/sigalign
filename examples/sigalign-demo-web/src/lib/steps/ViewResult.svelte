<script lang="ts">
  import type { AlignmentResult, Row } from "../../wasm/sigalign_demo_wasm";
  import { JsonView } from '@zerodevx/svelte-json-view';
  export let alignmentResult: AlignmentResult;

  enum ParsedType {
    Json,
    Table,
  }

  let parsedType: ParsedType = null;
  let parsedJsonObj: any = null;
  let parsedTable: Row[] = null;

  function resToJson() {
    parsedType = ParsedType.Json;
    let jsonString = alignmentResult.to_json();
    parsedJsonObj = JSON.parse(jsonString);
  }
  function resToTable() {
    parsedType = ParsedType.Table;
    parsedTable = alignmentResult.to_table();
  }
  function clear() {
    parsedType = null;
    parsedJsonObj = null;
    parsedTable = null;
  }
</script>

<h3 class="header">🛠️ Result Viewer</h3>

<!-- Result Viewer -->
{#if alignmentResult === null}
  <div class="unprepared">
    Alignment result it not ready.
  </div>
{:else}
  {#if parsedType === ParsedType.Json}
    <div class="jsonviewer">
      <JsonView
        bind:json={parsedJsonObj}
      ></JsonView>
    </div>
  {:else if parsedType === ParsedType.Table}
    <div class="table">
      <i>Operation: M(Match), S(Substitution), I(Insertion), and D(Deletion).</i>
      <table>
        <thead>
          <tr>
            <th>Index</th>
            <th>Label</th>
            <th>Penalty</th>
            <th>Length</th>
            <th>QueryPosition</th>
            <th>RecordPosition</th>
            <th>Operations</th>
          </tr>
        </thead>
        <tbody>
          {#each parsedTable as row}
            <tr>
              <td>{row.index}</td>
              <td>{row.label}</td>
              <td>{row.penalty}</td>
              <td>{row.length}</td>
              <td>{row.qstart}-{row.qend}</td>
              <td>{row.rstart}-{row.rend}</td>
              <td>{row.ops}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
{/if}

<!-- Controller -->
{#if !(parsedType === null)}
  <div class="button-wrapper">
    <button class="default primary" on:click={clear}>Clear</button>
  </div>
{/if}
{#if !(alignmentResult === null)}
  <div class="button-wrapper">
    <button class="default primary" on:click={resToJson}>View Result as JSON</button>
  </div>
  <div class="button-wrapper">
    <button class="default primary" on:click={resToTable}>View Result as Table</button>
  </div>
{/if}

<style>
  div.unprepared {
    text-align: center;
  }
  div.jsonviewer {
    padding-left: 2rem;
  }
  div.table {
    position: relative;
    overflow: auto;
    font-size: 0.8rem;
  }
</style>