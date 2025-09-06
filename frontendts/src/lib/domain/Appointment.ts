import dayjs from "dayjs";

export default class Appointment {
	constructor(
		public patientId = 0,
		public date = dayjs(),
		public state = State.Booked,
		public exams: string[] | null = null,
	) {}
}

export enum State {
	InProgress,
	Booked,
	Completed,
	Canceled,
}
