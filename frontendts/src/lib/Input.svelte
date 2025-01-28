<script lang="ts">
  interface Props {
    id: string;
    name: string;
    value: string;
    readonly: boolean;
    _type: string;
    _class: string;
  }
  let { id, name, value, readonly, _type = "text", _class = "" }: Props = $props();

  console.log(`id: ${id}`)
  if (id == null || id.trim() == "") {
    let rnd = Math.floor(Math.random() * 1000);
    id = `input-${name}-${rnd}`;
  }
</script>

<div class={`history-input ${_class}`}>
  <label for={id}>
    {name}
  </label>
  <input id={id} placeholder={name} value={value} disabled={readonly} type={_type} />
  <!--on:input=on_input-->
</div>
