<script lang="ts">
import { requirementHint } from "$entities/profile";
import { ProfileField, type ProtectedAction, Sex } from "$shared/api";
import type { FieldErrors, ProfileDraft } from "../model/draft";

interface Props {
  /** Borrador compartido: se edita en el lugar. */
  draft: ProfileDraft;
  /** Datos a mostrar, en orden. */
  fields: ProfileField[];
  /** Datos obligatorios para la acción en curso (se marcan con *). */
  required?: ProfileField[];
  /** Acciones que vuelven obligatorio cada dato, para el texto de ayuda. */
  requiredFor?: Partial<Record<ProfileField, ProtectedAction[]>>;
  /** Datos que el usuario no puede editar (verificados). */
  readOnly?: ProfileField[];
  errors?: FieldErrors;
  /** Prefijo de los `id` para no repetirlos entre la página y la ventana. */
  idPrefix?: string;
}

let {
  draft = $bindable(),
  fields,
  required = [],
  requiredFor = {},
  readOnly = [],
  errors = {},
  idPrefix = "profile",
}: Props = $props();

const inputClass =
  "w-full rounded-md border px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none disabled:bg-gray-50 disabled:text-gray-500";

const isRequired = (field: ProfileField) => required.includes(field);
const isReadOnly = (field: ProfileField) =>
  readOnly.includes(field) || field === ProfileField.EMAIL;
const border = (field: ProfileField) =>
  errors[field] ? "border-red-400" : "border-gray-300";
const hint = (field: ProfileField) =>
  isRequired(field)
    ? "Obligatorio para continuar"
    : requirementHint(requiredFor[field] ?? []);
const id = (name: string) => `${idPrefix}-${name}`;
</script>

{#snippet label(field: ProfileField, forId: string, text: string)}
  <label class="block text-sm font-medium text-gray-700 mb-1" for={forId}>
    {text}{isRequired(field) ? " *" : ""}
  </label>
{/snippet}

{#snippet help(field: ProfileField)}
  {#if errors[field]}
    <p class="mt-1 text-xs text-red-600" role="alert">{errors[field]}</p>
  {:else if isReadOnly(field) && field !== ProfileField.EMAIL}
    <p class="mt-1 text-xs text-gray-500">Verificado. Para cambiarlo, acércate a recepción.</p>
  {:else}
    <p class="mt-1 text-xs text-gray-500" data-hint>{hint(field)}</p>
  {/if}
{/snippet}

<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
  {#each fields as field (field)}
    {#if field === ProfileField.EMAIL}
      <div class="sm:col-span-2">
        {@render label(field, id("email"), "Correo electrónico")}
        <input class="{inputClass} border-gray-300" disabled id={id("email")} type="email" value={draft.email} />
      </div>
    {:else if field === ProfileField.GIVEN_NAME}
      <div>
        {@render label(field, id("given-name"), "Nombres")}
        <input bind:value={draft.givenName} class="{inputClass} {border(field)}" id={id("given-name")} maxlength="100" type="text" />
        {@render help(field)}
      </div>
    {:else if field === ProfileField.FAMILY_NAME}
      <div>
        {@render label(field, id("family-name"), "Apellido paterno")}
        <input bind:value={draft.familyName} class="{inputClass} {border(field)}" id={id("family-name")} maxlength="50" type="text" />
        {@render help(field)}
      </div>
    {:else if field === ProfileField.SECOND_FAMILY_NAME}
      <div>
        {@render label(field, id("second-family-name"), "Apellido materno")}
        <input bind:value={draft.secondFamilyName} class="{inputClass} {border(field)}" id={id("second-family-name")} maxlength="50" type="text" />
        {@render help(field)}
      </div>
    {:else if field === ProfileField.PHONE}
      <div>
        {@render label(field, id("phone"), "Teléfono")}
        <input bind:value={draft.phone} class="{inputClass} {border(field)}" id={id("phone")} maxlength="20" placeholder="+51 999 888 777" type="tel" />
        {@render help(field)}
      </div>
    {:else if field === ProfileField.DOCUMENT}
      <div class="sm:col-span-2 grid grid-cols-3 gap-2">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1" for={id("document-type")}>Tipo</label>
          <select bind:value={draft.documentType} class="{inputClass} border-gray-300" disabled={isReadOnly(field)} id={id("document-type")}>
            <option value="DNI">DNI</option>
            <option value="CE">Carné de extranjería</option>
            <option value="PASSPORT">Pasaporte</option>
          </select>
        </div>
        <div class="col-span-2">
          {@render label(field, id("document"), "Número de documento")}
          <input bind:value={draft.documentNumber} class="{inputClass} {border(field)}" disabled={isReadOnly(field)} id={id("document")} maxlength="12" type="text" />
          {@render help(field)}
        </div>
      </div>
    {:else if field === ProfileField.BIRTH_DATE}
      <div>
        {@render label(field, id("birth-date"), "Fecha de nacimiento")}
        <input bind:value={draft.birthDate} class="{inputClass} {border(field)}" id={id("birth-date")} type="date" />
        {@render help(field)}
      </div>
    {:else if field === ProfileField.SEX}
      <div>
        {@render label(field, id("sex"), "Sexo")}
        <select bind:value={draft.sex} class="{inputClass} {border(field)}" id={id("sex")}>
          <option value={Sex.UNSPECIFIED}>Sin indicar</option>
          <option value={Sex.FEMALE}>Femenino</option>
          <option value={Sex.MALE}>Masculino</option>
          <option value={Sex.OTHER}>Otro</option>
        </select>
        {@render help(field)}
      </div>
    {:else if field === ProfileField.ADDRESS}
      <div class="sm:col-span-2">
        {@render label(field, id("address"), "Dirección")}
        <input bind:value={draft.address} class="{inputClass} {border(field)}" id={id("address")} maxlength="255" type="text" />
        {@render help(field)}
      </div>
    {:else if field === ProfileField.PROFESSIONAL_LICENSE}
      <div>
        {@render label(field, id("professional-body"), "Colegio profesional")}
        <select bind:value={draft.professionalBody} class="{inputClass} {border(field)}" disabled={isReadOnly(field)} id={id("professional-body")}>
          <option value="">Sin indicar</option>
          <option value="CMP">CMP (médicos)</option>
          <option value="COP">COP (odontólogos)</option>
          <option value="CEP">CEP (enfermeros)</option>
          <option value="CPSP">CPsP (psicólogos)</option>
        </select>
      </div>
      <div>
        {@render label(field, id("license-number"), "Número de colegiatura")}
        <input bind:value={draft.licenseNumber} class="{inputClass} {border(field)}" disabled={isReadOnly(field)} id={id("license-number")} maxlength="30" type="text" />
        {@render help(field)}
      </div>
    {:else if field === ProfileField.SPECIALTY}
      <div>
        {@render label(field, id("specialty"), "Especialidad")}
        <input bind:value={draft.specialty} class="{inputClass} {border(field)}" id={id("specialty")} maxlength="100" type="text" />
        {@render help(field)}
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for={id("rne")}>RNE</label>
        <input bind:value={draft.rne} class="{inputClass} border-gray-300" id={id("rne")} maxlength="30" type="text" />
      </div>
    {:else if field === ProfileField.CLINIC_TAX_ID}
      <div>
        {@render label(field, id("clinic-tax-id"), "RUC de la clínica")}
        <input bind:value={draft.clinicTaxId} class="{inputClass} {border(field)}" id={id("clinic-tax-id")} inputmode="numeric" maxlength="11" type="text" />
        {@render help(field)}
      </div>
    {:else if field === ProfileField.CLINIC_LEGAL_NAME}
      <div>
        {@render label(field, id("clinic-legal-name"), "Razón social")}
        <input bind:value={draft.clinicLegalName} class="{inputClass} {border(field)}" id={id("clinic-legal-name")} maxlength="200" type="text" />
        {@render help(field)}
      </div>
    {:else if field === ProfileField.CLINIC_TRADE_NAME}
      <div>
        {@render label(field, id("clinic-trade-name"), "Nombre comercial")}
        <input bind:value={draft.clinicTradeName} class="{inputClass} {border(field)}" id={id("clinic-trade-name")} maxlength="200" type="text" />
        {@render help(field)}
      </div>
    {:else if field === ProfileField.IPRESS_CODE}
      <div>
        {@render label(field, id("ipress-code"), "Código IPRESS")}
        <input bind:value={draft.ipressCode} class="{inputClass} {border(field)}" id={id("ipress-code")} maxlength="20" type="text" />
        {@render help(field)}
      </div>
    {:else if field === ProfileField.LICENSE_CONFIRMED}
      <p class="sm:col-span-2 text-sm text-amber-800 bg-amber-50 border border-amber-200 rounded-md p-3">
        La clínica aún debe confirmar tu colegiatura.
      </p>
    {/if}
  {/each}
</div>
