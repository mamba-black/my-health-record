import Address from "$lib/domain/Address";
import Allergies from "$lib/domain/Allergies";

export default class Patient {
  constructor(
    public id: string,
    public firstName: string,
    public lastName: string,
    public age: number,
    public secondLastName?: string,
    public email?: string,
    public address: Address = new Address(),
    public allergies: Allergies = new Allergies(),
  ) {}
}
