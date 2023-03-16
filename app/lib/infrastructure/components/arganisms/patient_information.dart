import 'package:age_calculator/age_calculator.dart';
import 'package:fhir/primitive_types/date.dart';
import 'package:fhir/r5/basic_types/fhir_extension.dart';
import 'package:fhir/r5/resource_types/base/individuals/individuals.dart';
import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';

import '../atoms/field.dart';

class PatientInformation extends StatelessWidget {
  static const SPACING = 35.0;
  Patient? patient;

  PatientInformation({this.patient, super.key});

  @override
  Widget build(BuildContext context) {
    var mediaQueryData = MediaQuery.of(context);
    var width = mediaQueryData.size.width;
    FhirExtension(valueString: "");
    var humanName = patient?.name?.elementAt(0);
    var address = patient?.address?.elementAt(0).text;
    var name = humanName?.given?.join(" ");
    var lastName = humanName?.family?.split(",");
    var father = lastName?.elementAt(0);
    var mother = lastName?.elementAt(1);
    var birthDate = patient?.birthDate?.value;
    var phone = patient?.telecom?.elementAt(0).value;
    String years = "";

    if (birthDate != null) {
      // years = DateTime.now().difference(birthDate);
      years = "${AgeCalculator.age(birthDate).years}";
    }

    return Padding(
      padding: const EdgeInsets.all(20),
      child: Form(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            nroHistory("40329659-2"),
            Text("Datos generales:", style: TextStyle(fontWeight: FontWeight.bold)),
            const Spacer(flex: 2),
            wrap(
              [
                Field("Nombre", TextEditingController(text: name), width - 40, width / 3 - 40, 150),
                Field("Apellido Paterno", TextEditingController(text: father), width - 40, width / 3 - 40, 150),
                Field("Apellido Materno", TextEditingController(text: mother), width - 40, width / 3 - 40, 150),
              ],
            ),
            const Spacer(flex: 2),
            wrap(
              [
                Field("Direccion", TextEditingController(text: address), width - 50, width - 50, 300),
              ],
            ),
            const Spacer(flex: 2),
            wrap(
              [
                Field("Edad", TextEditingController(text: years), width - 40, (width - 155) / 4, 100),
                Field("Sexo", TextEditingController(text: "Masculino"), width - 40, (width - 155) / 4, 100),
                Field("Celular", TextEditingController(text: phone), width - 40, (width - 155) / 4, 100),
                Field("Estado Civil", TextEditingController(text: "Casado"), width - 40, (width - 155) / 4, 100)
              ],
            ),
            const Spacer(),
            Text(
              "Antecedentes:",
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            const Spacer(),
            // ListView.builder(
            //     padding: const EdgeInsets.all(8),
            //     itemCount: 4,
            //     itemBuilder: (BuildContext context, int index) {
            //       return Container(
            //         height: 50,
            //         child: Center(child: Text('Entry ${index}')),
            //       );
            //     }
            // ),
            const Spacer(flex: 40),
          ],
        ),
      ),
    );
  }

  Widget wrap(List<Widget> children) {
    return Wrap(
      direction: Axis.horizontal,
      spacing: SPACING,
      runSpacing: SPACING,
      children: children,
    );
  }

  Widget nroHistory(String dni) {
    return Row(
      children: [
        const Spacer(),
        SizedBox(
          width: 92,
          height: 70,
          child: TextFormField(
            readOnly: true,
            controller: TextEditingController(text: dni),
            textAlign: TextAlign.right,
            style: TextStyle(fontWeight: FontWeight.bold),
            decoration: const InputDecoration(
              labelText: 'Nro de Historia',
            ),
          ),
        ),
      ],
    );
  }
}
