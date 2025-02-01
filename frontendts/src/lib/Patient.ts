import Address from "$lib/Address";

export default class Patient {
  id: string;
  firstName: string;
  lastName: string;
  secondLastName?: string;
  age?: number;
  email?: string;
  address: Address;

  constructor(
    id: string,
    firstName: string,
    lastName: string,
    age: number,
    secondLastName?: string,
    email?: string,
    address: Address = new Address(),
  ) {
    this.id = id;
    this.firstName = firstName;
    this.lastName = lastName;
    this.secondLastName = secondLastName;
    this.age = age;
    this.email = email;
    this.address = address;
  }
}
