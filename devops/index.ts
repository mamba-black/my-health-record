import * as aws from "@pulumi/aws";
import * as pulumi from '@pulumi/pulumi';

// pulumi config set aws:profile <profilename>
const config = new pulumi.Config('cloud');

let pocAkkaGrpcTargetGroup = new aws.lb.TargetGroup(
  'poc-akka-grpc',
  {
    name: 'poc-akka-grpc',
    port: 9090,
    protocol: 'TCP',
    protocolVersion: 'GRPC', // CREO QUE NO ES NECESARIO PARA TCP
    targetType: 'instance',
    vpcId: 'vpc-03be0acf376a67be8',
  }
)

const loadBalancer = config.requireSecret('loadbalancer')

let pocAkkaGrpcListener = new aws.lb.Listener(
  'poc-akka-grpc',
  {
    port: 9090,
    protocol: 'TCP',
    defaultActions: [
      {
        type: 'forward',
        targetGroupArn: pocAkkaGrpcTargetGroup.arn
      }
    ],
    loadBalancerArn: loadBalancer
  },
  { dependsOn: pocAkkaGrpcTargetGroup, parent: pocAkkaGrpcTargetGroup }
)

let akkaGrpcContainerDefinition: aws.ecs.ContainerDefinition = {
  name: 'akkaGrpc',
  image: '957838095201.dkr.ecr.us-east-1.amazonaws.com/miclaro/pocs:0.0.0-57-03350ccb-20210624-1808',
  cpu: 0,
  memoryReservation: 512,
  portMappings: [
    {
      containerPort: 9090,
      hostPort: 0,
      protocol: 'tcp',
    }
  ],
};

// aws --profile claro ecs describe-task-definition --task-definition arn:aws:ecs:us-east-1:111111111111:task-definition/task-iiiiiiiiiiiiiiiiiiiiiiiii:7
let pocAkkaGrpcTaskDefinition = new aws.ecs.TaskDefinition(
  'poc-akka-grpc-td',
  {
    containerDefinitions: JSON.stringify([akkaGrpcContainerDefinition]),
    family: 'poc-akka-grpc',
    requiresCompatibilities: ['EC2'],
  },
  { dependsOn: [ pocAkkaGrpcListener ], parent: pocAkkaGrpcListener }
)

// aws --profile claro ecs describe-services --cluster arn:aws:ecs:us-east-1:111111111111:cluster/CLUSTER-ECS-MICLARO-DEV --services arn:aws:ecs:us-east-1:111111111111:service/sms
let pocAkkaGrpcService = new aws.ecs.Service(
  'poc-akka-grpc-se',
  {
    name: 'poc-akka-grpc',
    taskDefinition: pocAkkaGrpcTaskDefinition.arn,
    desiredCount: 2,
    launchType: 'EC2',
    cluster: config.requireSecret('ecscluster'),
    loadBalancers: [{
      containerName: 'akkaGrpc',
      containerPort: 9090,
      // elbName: 'NLB-DESARROLLO-NEW',
      targetGroupArn: pocAkkaGrpcTargetGroup.arn,
    }],

  },
  { dependsOn: [ pocAkkaGrpcTaskDefinition, pocAkkaGrpcTargetGroup ], parent: pocAkkaGrpcTaskDefinition }
);







// logConfiguration: {
//   logDriver: 'awslogs',
//   options: {
//     'awslogs-group': '/ecs/miclaro-poc-akka-grpc',
//     'awslogs-region': 'us-east-1',
//     'awslogs-stream-prefix': 'ecs'
//   }
// }
// aws.ecr.getRepository({
//   name: 'miclaro/pocs',
//   registryId: '957838095201',
// })

// aws.ecr.Repository.get("miclaro/pocs", "", {registryId: '957838095201'})

// pulumi import aws:ecr/repository:Repository miclaro/pocs miclaro/pocs
// new aws.ecr.Repository(
//   'miclaro/pocs',
//   { name: 'miclaro/pocs' },
//   { import: 'arn:' }
// )


