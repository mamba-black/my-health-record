import * as aws from "@pulumi/aws";
import * as pulumi from '@pulumi/pulumi';

// pulumi config set aws:profile <profilename>
const config = new pulumi.Config('cloud');

const targetGroup = new aws.lb.TargetGroup(
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

const listener = new aws.lb.Listener(
  'poc-akka-grpc',
  {
    port: 9090,
    protocol: 'TCP',
    defaultActions: [
      {
        type: 'forward',
        targetGroupArn: targetGroup.arn
      }
    ],
    loadBalancerArn: loadBalancer
  },
  { dependsOn: targetGroup, parent: targetGroup }
)

const containerDefinition: aws.ecs.ContainerDefinition = {
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

// // new aws.autoscaling.Policy('', {
// //     autoscalingGroupName: undefined,
// //     metricAggregationType: 'Average'
// //
// //   }
// // )
//
// // const cluster = new aws.ecs.Cluster()
// // cluster.defaultCapacityProviderStrategies
// //




// const launchTemplate = new aws.ec2.LaunchTemplate(
//   "poc-akka-grpc",
//   {
//     name: 'poc-akka-grpc',
//     imageId: "ami-1a2b3c",
//     instanceType: "t2.micro",
//   }
// );
//
// // error: aws:autoscaling/group:Group resource 'poc-akka-grpc' has a problem: ExactlyOne: "launch_configuration":   one of `launch_configuration,launch_template,mixed_instances_policy` must be specified. Examine values at 'Group.LaunchConfiguration'.
// // error: aws:autoscaling/group:Group resource 'poc-akka-grpc' has a problem: ExactlyOne: "mixed_instances_policy": one of `launch_configuration,launch_template,mixed_instances_policy` must be specified. Examine values at 'Group.MixedInstancesPolicy'.
// const autoscalingGroup = new aws.autoscaling.Group(
//   'poc-akka-grpc',
//   {
//     minSize: 2,
//     maxSize: 6,
//     launchTemplate: {
//       name: 'poc-akka-grpc',
//     },
//     // launchConfiguration: launchConfiguration,
//     tags: [{
//       key: "AmazonECSManaged",
//       value: "",
//       propagateAtLaunch: true,
//     }]
//   }
// )
// // new aws.autoscaling.Policy()
//
// const capacityProvider = new aws.ecs.CapacityProvider(
//   'poc-akka-grpc',
//   {
//     autoScalingGroupProvider: {
//       autoScalingGroupArn: autoscalingGroup.arn,
//       managedTerminationProtection: "ENABLED",
//       managedScaling: {
//         maximumScalingStepSize: 1000,
//         minimumScalingStepSize: 1,
//         status: "ENABLED",
//         targetCapacity: 10,
//       },
//     },
//   }
// )

// aws --profile claro ecs describe-task-definition --task-definition arn:aws:ecs:us-east-1:111111111111:task-definition/task-iiiiiiiiiiiiiiiiiiiiiiiii:7
const taskDefinition = new aws.ecs.TaskDefinition(
  'poc-akka-grpc-td',
  {
    containerDefinitions: JSON.stringify([containerDefinition]),
    family: 'poc-akka-grpc',
    requiresCompatibilities: ['EC2'],
  },
  { dependsOn: [listener], parent: listener }
)

// aws --profile claro ecs describe-services --cluster arn:aws:ecs:us-east-1:111111111111:cluster/CLUSTER-ECS-MICLARO-DEV --services arn:aws:ecs:us-east-1:111111111111:service/sms
const service = new aws.ecs.Service(
  'poc-akka-grpc-se',
  {
    name: 'poc-akka-grpc',
    taskDefinition: taskDefinition.arn,
    desiredCount: 4,
    deploymentMinimumHealthyPercent: 100,
    deploymentMaximumPercent: 200,
    launchType: 'EC2',
    cluster: config.requireSecret('ecscluster'),
    loadBalancers: [{
      containerName: 'akkaGrpc',
      containerPort: 9090,
      targetGroupArn: targetGroup.arn,
    }],
    // capacityProviderStrategies: [
    //   {
    //     capacityProvider: capacityProvider.name,
    //     base: 2,
    //   }
    // ],
  },
  { dependsOn: [taskDefinition, targetGroup], parent: taskDefinition }
);


// // logConfiguration: {
// //   logDriver: 'awslogs',
// //   options: {
// //     'awslogs-group': '/ecs/miclaro-poc-akka-grpc',
// //     'awslogs-region': 'us-east-1',
// //     'awslogs-stream-prefix': 'ecs'
// //   }
// // }
// // aws.ecr.getRepository({
// //   name: 'miclaro/pocs',
// //   registryId: '957838095201',
// // })
//
// // aws.ecr.Repository.get("miclaro/pocs", "", {registryId: '957838095201'})
//
// // pulumi import aws:ecr/repository:Repository miclaro/pocs miclaro/pocs
// // new aws.ecr.Repository(
// //   'miclaro/pocs',
// //   { name: 'miclaro/pocs' },
// //   { import: 'arn:' }
// // )
//
//
