package medical.infrastructure.ui

import com.raquo.laminar.api.L.EventBus

package object command {
  type CommandBus = EventBus[Command]
}
