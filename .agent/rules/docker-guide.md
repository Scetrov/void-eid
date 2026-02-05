---
trigger: glob
globs: **/Dockerfile,**/Dockerfile.*,**/*.dockerfile,**/docker-compose*.yml,**/docker-compose*.yaml,**/compose*.yml,**/compose*.yaml
---

## Core Principles of Containerization

### **1. Immutability**

- **Principle:** Once a container image is built, it should not change. Any changes should result in a new image.
- **Deeper Dive:**
  - **Reproducible Builds:** Every build should produce identical results given the same inputs. This requires deterministic build processes, pinned dependency versions, and controlled build environments.
  - **Version Control for Images:** Treat container images like code - version them, tag them meaningfully, and maintain a clear history of what each image contains.
  - **Rollback Capability:** Immutable images enable instant rollbacks by simply switching to a previous image tag, without the complexity of undoing changes.
  - **Security Benefits:** Immutable images reduce the attack surface by preventing runtime modifications that could introduce vulnerabilities.
- **Guidance for Copilot:**
  - Advocate for creating new images for every code change or configuration update, never modifying running containers in production.
  - Recommend using semantic versioning for image tags (e.g., `v1.2.3`, `latest` for development only).
  - Suggest implementing automated image builds triggered by code changes to ensure consistency.
  - Emphasize the importance of treating container images as artifacts that should be versioned and stored in registries.
- **Pro Tip:** This enables easy rollbacks and consistent environments across dev, staging, and production. Immutable images are the foundation of reliable deployments.

### **2. Portability**

- **Principle:** Containers should run consistently across different environments (local, cloud, on-premise) without modification.
- **Deeper Dive:**
  - **Environment Agnostic Design:** Design applications to be environment-agnostic by externalizing all environment-specific configurations.
  - **Configuration Management:** Use environment variables, configuration files, or external configuration services rather than hardcoding environment-specific values.
  - **Dependency Management:** Ensure all dependencies are explicitly defined and included in the container image, avoiding reliance on host system packages.
  - **Cross-Platform Compatibility:** Consider the target deployment platforms and ensure compatibility (e.g., ARM vs x86, different Linux distributions).
- **Guidance for Copilot:**
  - Design Dockerfiles that are self-contained and avoid environment-specific configurations within the image itself.
  - Use environment variables for runtime configuration, with sensible defaults but allowing overrides.
  - Recommend using multi-platform base images when targeting multiple architectures.
  - Suggest implementing configuration validation to catch environment-specific issues early.
- **Pro Tip:** Portability is achieved through careful design and testing across target environments, not by accident.

### **3. Isolation**

- **Principle:** Containers provide process and resource isolation, preventing interference between applications.
- **Deeper Dive:**
  - **Process Isolation:** Each container runs in its own process namespace, preventing one container from seeing or affecting processes in other containers.
  - **Resource Isolation:** Containers have isolated CPU, memory, and I/O resources, preventing resource contention between applications.
  - **Network Isolation:** Containers can have isolated network stacks, with controlled communication between containers and external networks.
  - **Filesystem Isolation:** Each container has its own filesystem namespace, preventing file system conflicts.
- **Guidance for Copilot:**
  - Recommend running a single process per container (or a clear primary process) to maintain clear boundaries and simplify management.
  - Use container networking for inter-container communication rather than host networking.
  - Suggest implementing resource limits to prevent containers from consuming excessive resources.
  - Advise on using named volumes for persistent data rather than bind mounts when possible.
- **Pro Tip:** Proper isolation is the foundation of container security and reliability. Don't break isolation for convenience.

### **4. Efficiency & Small Images**

- **Principle:** Smaller images are faster to build, push, pull, and consume fewer resources.
- **Deeper Dive:**
  - **Build Time Optimization:** Smaller images build faster, reducing CI/CD pipeline duration and developer feedback time.
  - **Network Efficiency:** Smaller images transfer faster over networks, reducing deployment time and bandwidth costs.
  - **Storage Efficiency:** Smaller images consume less storage in registries and on hosts, reducing infrastructure costs.
  - **Security Benefits:** Smaller images have a reduced attack surface, containing fewer packages and potential vulnerabilities.
- **Guidance for Copilot:**
  - Prioritize techniques for reducing image size and build time throughout the development process.
  - Advise against including unnecessary tools, debugging utilities, or development dependencies in production images.
  - Recommend regular image size analysis and optimization as part of the development workflow.
  - Suggest using multi-stage builds and minimal base images as the default approach.
- **Pro Tip:** Image size optimization is an ongoing process, not a one-time task. Regularly review and optimize your images.
