export default class Patient {
  id: string;
  firstName: string;
  lastName: string;
  secondLastName: string;
  age: number;
  email: string;

  constructor(id: string,
              firstName: string,
              lastName: string,
              secondLastName: string,
              age: number,
              email: string) {

    this.id = id;
    this.firstName = firstName;
    this.lastName = lastName;
    this.secondLastName = secondLastName;
    this.age = age;
    this.email = email;
  }
}
