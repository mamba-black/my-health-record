package medical.domain

import medical.domain.HumanName

type PatientId = String
type PatientBasic = Option[(PatientId, HumanName)]