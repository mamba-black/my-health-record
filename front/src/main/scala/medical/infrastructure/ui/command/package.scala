package medical.infrastructure.ui.command

import com.raquo.laminar.api.L.EventBus

type CommandBus = EventBus[Command]
