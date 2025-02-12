<script lang="ts">
  interface Props {
    id: string;
    name: string;
    value?: string;
    readonly: boolean;
    _type?: string;
    required?: boolean;
    _class?: string;
  }

  let {
    id,
    name,
    value = $bindable(),
    readonly,
    _type = "text",
    required = false,
    _class = "",
  }: Props = $props();

  if (id == null || id.trim() == "") {
    let rnd = Math.floor(Math.random() * 1000);
    id = `input-${name}-${rnd}`;
  }
</script>

<div class={`history-input ${_class}`}>
  <label for={id}>
    {name}
  </label>
  <input id={id} placeholder={name} bind:value={value} disabled={readonly} type={_type} required={required} />
  <!--on:input=on_input-->
</div>
