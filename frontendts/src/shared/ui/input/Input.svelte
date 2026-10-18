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

const computedId = $derived(
  id == null || id.trim() === ""
    ? `input-${name}-${Math.floor(Math.random() * 1000)}`
    : id,
);
</script>

<div class={`history-input ${_class}`}>
  <label for={computedId}>
    {name}
  </label>
  <input
    bind:value={value}
    disabled={readonly}
    id={computedId}
    placeholder={name}
    required={required}
    type={_type}
  />
</div>
