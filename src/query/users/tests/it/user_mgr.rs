// Copyright 2021 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use common_base::base::tokio;
use common_exception::ErrorCode;
use common_exception::Result;
use common_grpc::RpcClientConf;
use common_meta_app::principal::AuthInfo;
use common_meta_app::principal::AuthType;
use common_meta_app::principal::GrantObject;
use common_meta_app::principal::PasswordHashMethod;
use common_meta_app::principal::UserGrantSet;
use common_meta_app::principal::UserIdentity;
use common_meta_app::principal::UserInfo;
use common_meta_app::principal::UserPrivilegeSet;
use common_meta_app::principal::UserPrivilegeType;
use common_users::UserApiProvider;
use pretty_assertions::assert_eq;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_user_manager() -> Result<()> {
    let conf = RpcClientConf::default();
    let user_mgr = UserApiProvider::try_create_simple(conf).await?;

    let tenant = "test";
    let username = "test-user1";
    let hostname = "%";
    let pwd = "test-pwd";

    let auth_info = AuthInfo::Password {
        hash_value: Vec::from(pwd),
        hash_method: PasswordHashMethod::Sha256,
    };

    // add user hostname.
    {
        let user_info = UserInfo::new(username, hostname, auth_info.clone());
        user_mgr.add_user(tenant, user_info, false).await?;
    }

    // add user hostname again, error.
    {
        let user_info = UserInfo::new(username, hostname, auth_info.clone());
        let res = user_mgr.add_user(tenant, user_info, false).await;
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().code(), ErrorCode::USER_ALREADY_EXISTS);
    }

    // add user hostname again, ok.
    {
        let user_info = UserInfo::new(username, hostname, auth_info.clone());
        user_mgr.add_user(tenant, user_info, true).await?;
    }

    // get all users.
    {
        let users = user_mgr.get_users(tenant).await?;
        assert_eq!(1, users.len());
        assert_eq!(pwd.as_bytes(), users[0].auth_info.get_password().unwrap());
    }

    // get user hostname.
    {
        let user_info = user_mgr
            .get_user(tenant, UserIdentity::new(username, hostname), hostname)
            .await?;
        assert_eq!(hostname, user_info.hostname);
        assert_eq!(pwd.as_bytes(), user_info.auth_info.get_password().unwrap());
    }

    // drop.
    {
        user_mgr
            .drop_user(tenant, UserIdentity::new(username, hostname), false)
            .await?;
        let users = user_mgr.get_users(tenant).await?;
        assert_eq!(0, users.len());
    }

    // repeat drop same user not with if exist.
    {
        let res = user_mgr
            .drop_user(tenant, UserIdentity::new(username, hostname), false)
            .await;
        assert!(res.is_err());
    }

    // repeat drop same user with if exist.
    {
        let res = user_mgr
            .drop_user(tenant, UserIdentity::new(username, hostname), true)
            .await;
        assert!(res.is_ok());
    }

    // grant privileges
    {
        let user_info: UserInfo = UserInfo::new(username, hostname, auth_info.clone());
        user_mgr.add_user(tenant, user_info.clone(), false).await?;
        let old_user = user_mgr
            .get_user(tenant, user_info.identity(), hostname)
            .await?;
        assert_eq!(old_user.grants, UserGrantSet::empty());

        let mut add_priv = UserPrivilegeSet::empty();
        add_priv.set_privilege(UserPrivilegeType::Set);
        user_mgr
            .grant_privileges_to_user(tenant, user_info.identity(), GrantObject::Global, add_priv)
            .await?;
        let new_user = user_mgr
            .get_user(tenant, user_info.identity(), hostname)
            .await?;
        assert!(
            new_user
                .grants
                .verify_privilege(&GrantObject::Global, vec![UserPrivilegeType::Set])
        );
        assert!(
            !new_user
                .grants
                .verify_privilege(&GrantObject::Global, vec![UserPrivilegeType::Create])
        );
        user_mgr
            .drop_user(tenant, new_user.identity(), true)
            .await?;
    }

    // revoke privileges
    {
        let user_info: UserInfo = UserInfo::new(username, hostname, auth_info.clone());
        user_mgr.add_user(tenant, user_info.clone(), false).await?;
        user_mgr
            .grant_privileges_to_user(
                tenant,
                user_info.identity(),
                GrantObject::Global,
                UserPrivilegeSet::all_privileges(),
            )
            .await?;
        let user_info = user_mgr
            .get_user(tenant, user_info.identity(), hostname)
            .await?;
        assert_eq!(user_info.grants.entries().len(), 1);

        user_mgr
            .revoke_privileges_from_user(
                tenant,
                user_info.identity(),
                GrantObject::Global,
                UserPrivilegeSet::all_privileges(),
            )
            .await?;
        let user_info = user_mgr
            .get_user(tenant, user_info.identity(), hostname)
            .await?;
        assert_eq!(user_info.grants.entries().len(), 0);
        user_mgr
            .drop_user(tenant, user_info.identity(), true)
            .await?;
    }

    // alter.
    {
        let user = "test";
        let hostname = "%";
        let pwd = "test";
        let auth_info = AuthInfo::Password {
            hash_value: Vec::from(pwd),
            hash_method: PasswordHashMethod::Sha256,
        };
        let user_info: UserInfo = UserInfo::new(user, hostname, auth_info.clone());
        user_mgr.add_user(tenant, user_info.clone(), false).await?;

        let old_user = user_mgr
            .get_user(tenant, user_info.identity(), hostname)
            .await?;
        assert_eq!(old_user.auth_info.get_password().unwrap(), Vec::from(pwd));

        // alter both password & password_type
        let new_pwd = "test1";
        let auth_info = AuthInfo::Password {
            hash_value: Vec::from(new_pwd),
            hash_method: PasswordHashMethod::Sha256,
        };
        user_mgr
            .update_user(tenant, user_info.identity(), Some(auth_info), None)
            .await?;
        let new_user = user_mgr
            .get_user(tenant, user_info.identity(), hostname)
            .await?;
        assert_eq!(
            new_user.auth_info.get_password().unwrap(),
            Vec::from(new_pwd)
        );
        assert_eq!(
            new_user.auth_info.get_password_type().unwrap(),
            PasswordHashMethod::Sha256
        );

        // alter password only
        let new_new_pwd = "test2";
        let auth_info = AuthInfo::Password {
            hash_value: Vec::from(new_new_pwd),
            hash_method: PasswordHashMethod::Sha256,
        };
        user_mgr
            .update_user(tenant, user_info.identity(), Some(auth_info.clone()), None)
            .await?;
        let new_new_user = user_mgr
            .get_user(tenant, user_info.identity(), hostname)
            .await?;
        assert_eq!(
            new_new_user.auth_info.get_password().unwrap(),
            Vec::from(new_new_pwd)
        );

        let not_exist = user_mgr
            .update_user(
                tenant,
                UserIdentity::new("user", hostname),
                Some(auth_info.clone()),
                None,
            )
            .await;
        // ErrorCode::UnknownUser
        assert_eq!(not_exist.err().unwrap().code(), 2201)
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_user_manager_with_root_user() -> Result<()> {
    let conf = RpcClientConf::default();
    let user_mgr = UserApiProvider::try_create_simple(conf).await?;

    let tenant = "test";
    let username1 = "default";
    let username2 = "root";

    let hostname1 = "%";
    let client_ip = "127.0.0.1";
    let client_ip2 = "192.168.0.1";

    // Get user via username `default` and client_ip `127.0.0.1`.
    {
        let user = user_mgr
            .get_user(tenant, UserIdentity::new(username1, hostname1), client_ip)
            .await?;
        assert_eq!(user.name, username1);
        assert_eq!(user.hostname, hostname1);
        assert_eq!(user.auth_info, AuthInfo::None);
        assert!(user.grants.verify_privilege(&GrantObject::Global, vec![
            UserPrivilegeType::Create,
            UserPrivilegeType::Select,
            UserPrivilegeType::Insert,
            UserPrivilegeType::Super
        ]));
    }

    // Get user via username `default` and client_ip `192.168.0.1` will be denied.
    {
        let res = user_mgr
            .get_user(tenant, UserIdentity::new(username1, hostname1), client_ip2)
            .await;
        assert!(res.is_err());
        assert_eq!(
            "Code: 2201, Text = only accept root from 127.0.0.1, current: 'default'@'192.168.0.1'.",
            res.err().unwrap().to_string()
        );
    }

    // Get user via username `root` and client_ip `127.0.0.1`.
    {
        let user = user_mgr
            .get_user(tenant, UserIdentity::new(username2, hostname1), client_ip)
            .await?;
        assert_eq!(user.name, username2);
        assert_eq!(user.hostname, hostname1);
        assert_eq!(user.auth_info, AuthInfo::None);
        assert!(user.grants.verify_privilege(&GrantObject::Global, vec![
            UserPrivilegeType::Create,
            UserPrivilegeType::Select,
            UserPrivilegeType::Insert,
            UserPrivilegeType::Super
        ]));
    }

    // Get user via username `root` and client_ip `192.168.0.1` will be denied.
    {
        let res = user_mgr
            .get_user(tenant, UserIdentity::new(username2, hostname1), client_ip2)
            .await;
        assert!(res.is_err());
        assert_eq!(
            "Code: 2201, Text = only accept root from 127.0.0.1, current: 'root'@'192.168.0.1'.",
            res.err().unwrap().to_string()
        );
    }

    // create `root` user.
    {
        let user_info = UserInfo::new(username2, hostname1, AuthInfo::None);
        let res = user_mgr.add_user(tenant, user_info, false).await;
        assert!(res.is_err());
        assert_eq!(
            "Code: 2202, Text = User cannot be created with builtin user `root`.",
            res.err().unwrap().to_string()
        );
    }

    // drop `root` user.
    {
        let user_info = UserIdentity::new(username2, hostname1);
        let res = user_mgr.drop_user(tenant, user_info, false).await;
        assert!(res.is_err());
        assert_eq!(
            "Code: 2202, Text = Builtin user `root` cannot be dropped.",
            res.err().unwrap().to_string()
        );
    }

    // alter password for `root` user.
    {
        let new_pwd = "test1";
        let user_info = UserIdentity::new(username2, hostname1);
        let auth_info = AuthInfo::Password {
            hash_value: Vec::from(new_pwd),
            hash_method: PasswordHashMethod::DoubleSha1,
        };
        user_mgr
            .update_user(tenant, user_info.clone(), Some(auth_info), None)
            .await?;
        let new_user = user_mgr.get_user(tenant, user_info, client_ip).await?;
        assert_eq!(new_user.name, username2);
        assert_eq!(new_user.hostname, hostname1);
        assert_eq!(new_user.auth_info.get_type(), AuthType::DoubleSha1Password);
        assert_eq!(
            new_user.auth_info.get_password().unwrap(),
            Vec::from(new_pwd)
        );
        assert_eq!(
            new_user.auth_info.get_password_type().unwrap(),
            PasswordHashMethod::DoubleSha1
        );
    }

    // alter password for `root` user again.
    {
        let new_pwd = "test2";
        let user_info = UserIdentity::new(username2, hostname1);
        let auth_info = AuthInfo::Password {
            hash_value: Vec::from(new_pwd),
            hash_method: PasswordHashMethod::DoubleSha1,
        };
        user_mgr
            .update_user(tenant, user_info.clone(), Some(auth_info), None)
            .await?;
        let new_user = user_mgr.get_user(tenant, user_info, client_ip).await?;
        assert_eq!(new_user.name, username2);
        assert_eq!(new_user.hostname, hostname1);
        assert_eq!(new_user.auth_info.get_type(), AuthType::DoubleSha1Password);
        assert_eq!(
            new_user.auth_info.get_password().unwrap(),
            Vec::from(new_pwd)
        );
        assert_eq!(
            new_user.auth_info.get_password_type().unwrap(),
            PasswordHashMethod::DoubleSha1
        );
    }

    Ok(())
}
