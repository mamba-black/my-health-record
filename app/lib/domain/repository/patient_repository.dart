import 'package:medical_app/domain/patient.dart';

abstract class PatientRepository {
  List<Patient> findPatients(String name);
}
