import type { Page } from "@playwright/test";
import {
  expect,
  readAccount,
  seed,
  seedAppointment,
  signInAs,
  signInExisting,
  test,
  workerEmail,
} from "./support/fixtures";

// `scheduling-appointments`: reservar, cambiar y recibir citas desde la interfaz.
const CLINIC = "/c/e2e-clinica";

/** Médico propio de la prueba, con horario: las pruebas en paralelo no se pisan. */
function scheduledDoctor(kind: string, index: number) {
  const email = workerEmail(`agenda-dra-${kind}`, index);
  seed("practitioner-scheduled", email);
  return `Dra ${email.split("@")[0]} Prueba`;
}

const doctorEmail = (kind: string, index: number) => workerEmail(`agenda-dra-${kind}`, index);

async function pickFirstSlot(page: Page, doctor: string) {
  await page.getByRole("tab", { name: doctor }).click();
  const slot = page.getByTestId("slot-picker").locator("button[data-slot]").first();
  await expect(slot).toBeVisible();
  const start = await slot.getAttribute("data-slot");
  await slot.click();
  return start;
}

const uniqueDni = () =>
  `${String(Date.now()).slice(-6)}${String(test.info().parallelIndex).padStart(2, "0")}`;

// Escenario de `my-health-record-c2p`: datos faltantes al confirmar una cita.
test("Paciente sin documento reserva y completa sus datos sin volver a elegir el cupo", async ({
  page,
}, testInfo) => {
  const doctor = scheduledDoctor("doc", testInfo.parallelIndex);
  const email = workerEmail("agenda-paciente", testInfo.parallelIndex);
  await signInAs(page, "patient-no-document", email);
  await page.goto(CLINIC);
  await pickFirstSlot(page, doctor);

  const dialog = page.getByRole("dialog");
  await expect(dialog).toContainText("Para confirmar tu cita necesitamos estos datos");
  await page.locator("#requirement-document").fill(uniqueDni());
  await dialog.getByRole("button", { name: "Completar" }).click();

  await expect(page.getByText("Tu cita quedó confirmada.")).toBeVisible();
  expect(readAccount(email).document_value).toBeTruthy();
});

test("Cancelar libera el cupo y reprogramar mueve la cita", async ({ page }, testInfo) => {
  const doctor = scheduledDoctor("cambios", testInfo.parallelIndex);
  await signInAs(page, "patient-complete", workerEmail("agenda-cambios", testInfo.parallelIndex));
  await page.goto(CLINIC);
  const start = await pickFirstSlot(page, doctor);
  await expect(page.getByText("Tu cita quedó confirmada.")).toBeVisible();

  await page.goto("/appointments");
  const item = page.locator("li[data-appointment]").first();
  await item.getByRole("button", { name: "Cambiar horario" }).click();
  const target = item.getByTestId("slot-picker").locator("button[data-slot]").nth(1);
  const newStart = await target.getAttribute("data-slot");
  await target.click();
  await expect(page.getByRole("status")).toContainText("Tu cita pasó al");
  expect(newStart).not.toBe(start);

  await page.locator("li[data-appointment]").first().getByRole("button", { name: "Cancelar" }).click();
  await page.getByRole("button", { name: "Sí, cancelar" }).click();
  await expect(page.getByRole("status")).toContainText("Cancelaste tu cita");
  await expect(page.locator("li[data-appointment] [data-status]").first()).toHaveText("Cancelada");

  // El cupo vuelve a verse libre en la página pública.
  await page.goto(CLINIC);
  await page.getByRole("tab", { name: doctor }).click();
  await expect(page.locator(`button[data-slot="${newStart}"]`)).toBeVisible();
});

test("Recepción reserva por un paciente sin cuenta y registra su llegada", async ({
  page,
}, testInfo) => {
  const doctor = scheduledDoctor("recepcion", testInfo.parallelIndex);
  await signInAs(page, "receptionist-active", workerEmail("agenda-recepcion", testInfo.parallelIndex));
  await expect(page.getByRole("link", { name: "Agenda de la clínica" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Mi agenda" })).toBeHidden();
  await page.goto("/clinic-agenda");

  const form = page.getByTestId("book-for-patient");
  await form.getByLabel("Nombres del paciente").fill("Rosa");
  await form.getByLabel("Apellido paterno del paciente").fill("Quispe");
  await form.getByLabel("DNI del paciente").fill(uniqueDni());
  await form.getByLabel("Teléfono del paciente").fill("987654321");
  await form.getByLabel("Médico").selectOption({ label: doctor });
  await form.locator("button[data-slot]").first().click();
  await expect(form.getByRole("status")).toContainText("Reservaste para Rosa Quispe");
});

test("El médico publica su horario y ve el enlace de su clínica", async ({ page }, testInfo) => {
  const email = workerEmail("agenda-medico", testInfo.parallelIndex);
  await signInAs(page, "practitioner-scheduled", email);
  await expect(page.getByRole("link", { name: "Mi agenda" })).toBeVisible();
  await page.goto("/schedule");
  const editor = page.getByTestId("schedule-editor");
  await expect(editor.locator('[data-weekday="1"] input[type="time"]').first()).toHaveValue("00:00");
  await editor.getByRole("button", { name: "Guardar horario" }).click();
  await expect(page.getByRole("status")).toContainText("Guardaste tu horario");
  await page.getByRole("link", { name: "/c/e2e-clinica" }).click();
  await page.getByRole("tab", { name: `Dra ${email.split("@")[0]}` }).click();
  await expect(page.getByTestId("slot-picker").locator("button[data-slot]").first()).toBeVisible();
});

test("Recepción registra la llegada con el DNI y el documento queda verificado", async ({
  page,
  browser,
}, testInfo) => {
  scheduledDoctor("llegada", testInfo.parallelIndex);
  const patientEmail = workerEmail("agenda-llega", testInfo.parallelIndex);
  seed("patient-complete", patientEmail);
  const { appointmentId } = seedAppointment(
    doctorEmail("llegada", testInfo.parallelIndex),
    "account",
    patientEmail,
  );

  await signInAs(page, "receptionist-active", workerEmail("agenda-rec-llega", testInfo.parallelIndex));
  await page.goto("/clinic-agenda");
  const item = page.locator(`li[data-appointment="${appointmentId}"]`);
  await item.getByLabel("Documento del paciente").selectOption({ label: "Vi el documento físico y coincide" });
  await item.getByRole("button", { name: "Llegó" }).click();
  await expect(page.getByRole("status")).toContainText("el documento quedó verificado");
  expect(readAccount(patientEmail).document_verified).toBe(true);

  // El paciente ve su documento como verificado y no lo puede editar.
  const patientPage = await (await browser.newContext()).newPage();
  await signInExisting(patientPage, patientEmail);
  await patientPage.goto("/complete-profile");
  await expect(patientPage.locator("#profile-document")).toBeDisabled();
});

test("Recepción vincula un paciente registrado sin cuenta y él ve su cita anterior", async ({
  page,
  browser,
}, testInfo) => {
  scheduledDoctor("vinculo", testInfo.parallelIndex);
  const patientEmail = workerEmail("agenda-vincula", testInfo.parallelIndex);
  seed("patient-complete", patientEmail);
  const { appointmentId } = seedAppointment(
    doctorEmail("vinculo", testInfo.parallelIndex),
    "walk-in",
    patientEmail,
  );

  await signInAs(page, "receptionist-active", workerEmail("agenda-rec-vincula", testInfo.parallelIndex));
  await page.goto("/clinic-agenda");
  const item = page.locator(`li[data-appointment="${appointmentId}"]`);
  await item.getByLabel("Documento del paciente").selectOption({ label: "Vi el documento físico y coincide" });
  await item.getByRole("button", { name: "Llegó" }).click();
  const suggestions = page.getByTestId("link-suggestions");
  await expect(suggestions).toContainText(patientEmail);
  await suggestions.getByRole("button", { name: "Vi el documento físico: vincular" }).click();
  await expect(page.getByRole("status")).toContainText("Vinculaste al paciente");

  const patientPage = await (await browser.newContext()).newPage();
  await signInExisting(patientPage, patientEmail);
  await patientPage.goto("/appointments");
  await expect(patientPage.locator(`li[data-appointment="${appointmentId}"]`)).toBeVisible();
});
