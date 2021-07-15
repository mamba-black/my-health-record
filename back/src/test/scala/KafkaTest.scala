import akka.actor.typed.scaladsl.Behaviors
import akka.actor.typed.{ActorSystem, Behavior}
import akka.kafka._
import akka.kafka.scaladsl._
import akka.stream.scaladsl.{Keep, Sink, Source}
import akka.stream.{FlowMonitor, FlowMonitorState}
import com.typesafe.config.ConfigFactory
import org.apache.kafka.clients.producer.ProducerRecord
import org.apache.kafka.common.serialization.{StringDeserializer, StringSerializer}
import scribe._

import java.time.ZonedDateTime
import scala.concurrent.duration.DurationInt

object KafkaTest {

  sealed case class KafjaTestCommand()

  def apply(): Behavior[KafjaTestCommand] =
    Behaviors.setup { context =>
      context.log.info("Begin Main")
      Behaviors.same
    }

  def main(args: Array[String]): Unit = {
    val applicationConfig = s"""
                               |akka.kafka.consumer {
                               |  kafka-clients {
                               |    bootstrap.servers = "PLAINTEXT://0.0.0.0:29092,PLAINTEXT_HOST://0.0.0.0:9092"
                               |  }
                               |}
                               |akka.kafka.producer {
                               |  kafka-clients {
                               |    bootstrap.servers = "PLAINTEXT://0.0.0.0:29092,PLAINTEXT_HOST://0.0.0.0:9092"
                               |  }
                               |}
                               |""".stripMargin
    info(applicationConfig)
    val config = ConfigFactory.parseString(applicationConfig).resolve()

    implicit val system: ActorSystem[KafjaTestCommand] = ActorSystem(KafkaTest(), "Test", config)
    system ! KafjaTestCommand()

    {
      val consumerConfig = system.settings.config.getConfig("akka.kafka.consumer")
      val consumerSettings =
        ConsumerSettings(consumerConfig, new StringDeserializer, new StringDeserializer).withGroupId("test-scala")
      val consumer = Consumer
        .committableSource(consumerSettings, Subscriptions.topics("topic1", "topic2"))

      val (flowMonitor, _) = consumer
        .monitorMat(Keep.right)
        .toMat(Sink.foreach(cm => info(s"$cm")))(Keep.both)
        .run()
      printMonitorState(flowMonitor)
      Source
        .tick(200.millis, 10.seconds, "")
        .runForeach(_ => printMonitorState(flowMonitor))

      consumer.run()
    }
    {
      val producerConfig = system.settings.config.getConfig("akka.kafka.producer")
      val producerSettings = ProducerSettings(producerConfig, new StringSerializer, new StringSerializer)
      val producer = Producer.plainSink(producerSettings)
      Source
        .tick(200.millis, 2.seconds, "")
        .map(_ => {
          val producerRecord = new ProducerRecord[String, String]("topic1", ZonedDateTime.now().toString)
          info(s"producerRecord: $producerRecord")
          producerRecord
        })
        .runWith(producer)
      info("Fin???")
    }

    ()
  }

  def printMonitorState(flowMonitor: FlowMonitor[ConsumerMessage.CommittableMessage[String, String]]): Unit =
    flowMonitor.state match {
      case FlowMonitorState.Initialized =>
        info("Stream is initialized but hasn't processed any element")
      case FlowMonitorState.Received(msg) =>
        info(s"Last element processed: $msg")
      case FlowMonitorState.Failed(cause) =>
        info(s"Stream failed with cause $cause")
      case FlowMonitorState.Finished => info(s"Stream completed already")
    }

}
