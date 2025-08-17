import Address from "$lib/domain/Address";
import MedicalConditions from "$lib/domain/MedicalConditions";

export default class Patient {
  constructor(
    public id: string,
    public firstName: string,
    public lastName: string,
    public age: number,
    public secondLastName?: string,
    public phone?: string,
    public email?: string,
    public address: Address = new Address(),
    public medicalConditions: MedicalConditions = new MedicalConditions(),
  ) {
  }
}
