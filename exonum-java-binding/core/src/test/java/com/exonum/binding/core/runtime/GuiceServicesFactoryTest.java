package com.exonum.binding.core.runtime;

import static org.assertj.core.api.Assertions.assertThat;
import static org.junit.jupiter.api.Assertions.assertThrows;

import com.exonum.binding.core.service.AbstractServiceModule;
import com.exonum.binding.core.service.Service;
import com.google.inject.Guice;
import com.google.inject.Injector;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class GuiceServicesFactoryTest {

  private static final String TEST_NAME = "test_service_instance";
  private static final int TEST_ID = 17;
  private GuiceServicesFactory factory;

  @BeforeEach
  void setUp() {
    // todo: if we designate some abstraction as 'provided' by the framework, test
    //   that they can be injected into the service instance
    Injector frameworkInjector = Guice.createInjector();
    factory = new GuiceServicesFactory(frameworkInjector);
  }

  @Test
  void createService() {
    ServiceArtifactId artifactId = ServiceArtifactId.parseFrom("com.acme:foo-service:1.0.0");
    LoadedServiceDefinition serviceDefinition = LoadedServiceDefinition
        .newInstance(artifactId, TestServiceModule::new);
    ServiceInstanceSpec instanceSpec = ServiceInstanceSpec.newInstance(TEST_NAME,
        TEST_ID, artifactId);

    // Create the service
    ServiceWrapper service = factory.createService(serviceDefinition, instanceSpec);

    // Check the created service
    assertThat(service.getName()).isEqualTo(TEST_NAME);
    assertThat(service.getService()).isInstanceOf(TestService.class);
  }

  @Test
  void createServiceFailsIfNoServiceBindingsInModule() {
    ServiceArtifactId artifactId = ServiceArtifactId.parseFrom("com.acme:incomplete-service:1.0.0");
    LoadedServiceDefinition serviceDefinition = LoadedServiceDefinition
        .newInstance(artifactId, IncompleteServiceModule::new);
    ServiceInstanceSpec instanceSpec = ServiceInstanceSpec.newInstance(TEST_NAME,
        TEST_ID, artifactId);

    // Try to create the service
    Exception e = assertThrows(IllegalArgumentException.class,
        () -> factory.createService(serviceDefinition, instanceSpec));

    // Check the message indicates missing bindings
    // todo: If we make some other bindings **required**, add them here.
    assertThat(e).hasMessageContaining(Service.class.getSimpleName());
  }
}

class IncompleteServiceModule extends AbstractServiceModule {
  // Incomplete as it does not configure Service bindings
}