## ADDED Requirements

### Requirement: Permission Evaluation
The permission system SHALL evaluate permissions based on patterns and user roles.

#### Scenario: Exact Match Permission
- **WHEN** user has permission "repo:read" and checks permission "repo:read"
- **THEN** permission is granted

#### Scenario: Wildcard Permission
- **WHEN** user has permission "repo:*" and checks permission "repo:read"
- **THEN** permission is granted

#### Scenario: Permission Denial
- **WHEN** user has permission "repo:read" and checks permission "repo:write"
- **THEN** permission is denied

#### Scenario: Multiple Permissions
- **WHEN** user has permissions ["repo:read", "issue:write"] and checks permission "issue:write"
- **THEN** permission is granted

#### Scenario: Permission Inheritance
- **WHEN** user has permission "admin" and admin implies all permissions
- **THEN** all permission checks return true

### Requirement: Permission Patterns
Permissions SHALL support glob-style and regex patterns for flexible matching.

#### Scenario: Glob Pattern Match
- **WHEN** user has permission "repo:*:read" and checks permission "repo:myproject:read"
- **THEN** permission is granted

#### Scenario: Regex Pattern Match
- **WHEN** user has permission matching regex "^repo:.*:read$" and checks permission "repo:myproject:read"
- **THEN** permission is granted

#### Scenario: Complex Pattern
- **WHEN** user has permission "user:*:repo:*:read" and checks permission "user:alice:repo:myproject:read"
- **THEN** permission is granted

### Requirement: Permission Management
The system SHALL support granting, revoking, and listing permissions.

#### Scenario: Grant Permission
- **WHEN** granting permission "repo:read" to user
- **THEN** user gains the permission

#### Scenario: Revoke Permission
- **WHEN** revoking permission "repo:read" from user
- **THEN** user loses the permission

#### Scenario: List Permissions
- **WHEN** listing permissions for user
- **THEN** returns all permissions assigned to user

### Requirement: Default Permissions
The system SHALL provide reasonable default permissions for common roles.

#### Scenario: Admin Defaults
- **WHEN** user has role "admin"
- **THEN** user gets all permissions by default

#### Scenario: User Defaults
- **WHEN** user has role "user" with no explicit permissions
- **THEN** user gets basic permissions (repo:read, issue:read, etc.)

#### Scenario: Guest Defaults
- **WHEN** user has role "guest" with no explicit permissions
- **THEN** user gets minimal permissions (repo:read only)