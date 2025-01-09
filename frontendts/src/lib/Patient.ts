export default class Patient {
  id: string;
  name: string;
  age: number;
  email: string;

  constructor(id: string, name: string, age: number, email: string) {
    this.id = id;
    this.name = name;
    this.age = age;
    this.email = email;
  }
}
