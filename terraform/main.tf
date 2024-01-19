provider "google" {
  credentials = file("irio-final-project-4f3f8f86d140.json")
  project     = "irio-final-project"
  region      = "europe-central2"
}

terraform {
  required_version = ">= 1.7"
}

resource "google_container_cluster" "irio_cluster" {
  name     = "irio-cluster"
  location = "europe-central2-a"
  initial_node_count = 3

  node_config {
    machine_type = "n1-standard-2"  # Adjust the machine type based on your requirements
  }
}

# Stateful PostgreSQL Database
resource "google_sql_database_instance" "postgres_db" {
  name             = "postgres-instance"
  database_version = "POSTGRES_13"
  region           = "europe-central2"

  settings {
    tier = "db-f1-micro"  # Adjust the tier based on your requirements
  }
}

resource "google_sql_database" "postgres_database" {
  name     = "mydatabase"
  instance = google_sql_database_instance.postgres_db.name
}

# Stateless Notification Service
resource "kubernetes_deployment" "notification_service" {
  metadata {
    name      = "notification-service"
    namespace = "default"
  }

  spec {
    replicas = 3  # Adjust the replica count based on your requirements

    selector {
      match_labels = {
        app = "notification-service"
      }
    }

    template {
      metadata {
        labels = {
          app = "notification-service"
        }
      }

      spec {
        container {
          name  = "notification-service"
          image = "your-notification-service-image"  # Replace with your actual image name
        }
      }
    }
  }
}

# Stateless Healthcheck Service
resource "kubernetes_deployment" "healthcheck_service" {
  metadata {
    name      = "healthcheck-service"
    namespace = "default"
  }

  spec {
    replicas = 3  # Adjust the replica count based on your requirements

    selector {
      match_labels = {
        app = "healthcheck-service"
      }
    }

    template {
      metadata {
        labels = {
          app = "healthcheck-service"
        }
      }

      spec {
        container {
          name  = "healthcheck-service"
          image = "your-healthcheck-service-image"  # Replace with your actual image name
        }
      }
    }
  }
}

# Allow Notification and Healthcheck services to connect to the database
resource "kubernetes_service" "postgres_service" {
  metadata {
    name      = "postgres-service"
    namespace = "default"
  }

  spec {
    selector = {
      app = "notification-service"  # Use the labels of your notification service
    }

    port {
      protocol = "TCP"
      port     = 5432
      target_port = 5432
    }
  }
}

resource "kubernetes_service" "notification_service" {
  metadata {
    name      = "notification-service"
    namespace = "default"
  }

  spec {
    selector = {
      app = "notification-service"
    }

    port {
      protocol = "TCP"
      port     = 80
      target_port = 80
    }
  }
}

resource "kubernetes_service" "healthcheck_service" {
  metadata {
    name      = "healthcheck-service"
    namespace = "default"
  }

  spec {
    selector = {
      app = "healthcheck-service"
    }

    port {
      protocol = "TCP"
      port     = 80
      target_port = 80
    }
  }
}
