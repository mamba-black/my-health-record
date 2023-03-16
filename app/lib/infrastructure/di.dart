import '../domain/repository/patient_repository.dart';
import 'repository/patient_repository_impl.dart';

class DI {
  static PatientRepository patientRepository() {
    return PatientRepositoryImpl();
  }
}
