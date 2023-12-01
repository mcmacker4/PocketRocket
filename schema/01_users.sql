-- Users
create table if not exists users
(
    id            int primary key auto_increment comment "Unique user ID",
    username      varchar(255) unique not null comment "Username",
    password_hash varchar(255)        not null comment "Password has using Argon2id",
    created_at    timestamp default current_timestamp comment "Date and time when this row was inserted",
    modified_at   timestamp default current_timestamp on update current_timestamp comment "Date and time when this row was updated last time"
);

-- Roles
create table if not exists roles
(
    id          int primary key auto_increment comment "Unique user ID",
    name        varchar(255) not null comment "Role name",
    created_at  timestamp default current_timestamp comment "Date and time when this row was inserted",
    modified_at timestamp default current_timestamp on update current_timestamp comment "Date and time when this row was updated last time"
);

-- Relation between users and roles
create table if not exists role_of_user
(
    user_id    int not null comment "User ID",
    role_id    int not null comment "Role ID",
    created_at timestamp default current_timestamp comment "Date and time when this row was inserted",
    constraint role_of_user_pk primary key (user_id, role_id),
    constraint role_of_user_users_id_fk foreign key (user_id) references users (id),
    constraint role_of_user_roles_id_fk foreign key (role_id) references roles (id)
);

-- Sessions
create table if not exists sessions
(
    id         int primary key auto_increment comment "Unique session ID",
    user_id    int          not null comment "User ID",
    token      varchar(255) not null comment "Session token",
    created_at timestamp default current_timestamp comment "Date and time when this row was inserted",
    constraint sessions_users_id_fk foreign key (user_id) references users (id)
);
